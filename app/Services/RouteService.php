<?php

declare(strict_types=1);

namespace App\Services;

use App\Enums\LifetimeStatus;
use App\Enums\MassStatus;
use App\Http\Resources\SolarsystemResource;
use App\Models\Map;
use App\Models\MapConnection;
use App\Models\Solarsystem;
use App\Scopes\ConnectionSatisfiesMass;
use Closure;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Support\Facades\Cache;
use NicolasKion\SDE\Models\SolarsystemConnection;
use SplPriorityQueue;
use SplQueue;

final readonly class RouteService
{
    public const string MAP_CACHE_PATTERN = 'map_%d'; // Keep public for MapRoutesBecameOutdated listener

    private const string BASE_CONNECTIONS_CACHE_KEY = 'route_service_base_connections';

    private const int CACHE_TTL = 3600; // 1 hour

    private array $connections;

    public function __construct()
    {
        $this->connections = $this->getConnections();
    }

    /**
     * Find the fastest route with session-based system exclusions
     */
    public function findRoute(int $fromId, int $toId, RouteOptions $options): array
    {
        $connections = $this->prepareConnections($options);
        $ignoredSystems = $options->ignoredSystems ?? [];

        // Find the fastest route avoiding ignored systems
        $path = $this->findFastestPath($fromId, $toId, $connections, $ignoredSystems);

        if ($path === []) {
            return [];
        }

        // Enrich with solarsystem data
        return $this->enrichRouteWithSolarsystemData($path);
    }

    /**
     * Find the closest systems matching a given condition
     */
    public function findClosestSystems(int $fromId, RouteOptions $options, Closure $condition, int $limit = 15): array
    {
        $connections = $this->prepareConnections($options);
        $ignoredSystems = $options->ignoredSystems ?? [];

        // Get all systems matching the condition
        $targetSystems = Solarsystem::query()
            ->where($condition)
            ->pluck('id')
            ->toArray();

        if ($targetSystems === []) {
            return [];
        }

        // Find distances to all target systems
        $distances = $this->findDistancesToMultipleTargets($fromId, $targetSystems, $connections, $ignoredSystems);

        // Sort by distance and take the closest ones
        asort($distances);
        $closestSystems = array_slice($distances, 0, $limit, true);

        // Enrich with solarsystem data
        $solarsystemIds = array_keys($closestSystems);
        $solarsystems = Solarsystem::query()
            ->with([
                'sovereignty' => ['alliance', 'corporation', 'faction'],
                'wormholeSystem.effect',
                'constellation',
                'region',
            ])
            ->whereIn('id', $solarsystemIds)
            ->get()
            ->keyBy('id');

        return collect($closestSystems)
            ->map(fn (int $distance, int $id): array => [
                'solarsystem' => $solarsystems->get($id)?->toResource(SolarsystemResource::class),
                'distance' => $distance,
            ])
            ->filter(fn (array $item): bool => $item['solarsystem'] !== null)
            ->values()
            ->toArray();
    }

    private function getConnections(): array
    {
        return Cache::remember(self::BASE_CONNECTIONS_CACHE_KEY, self::CACHE_TTL, static fn (): array => SolarsystemConnection::query()
            ->whereDoesntHaveRelation('fromSolarsystem', 'name', '=', 'Zarzakh')
            ->select('from_solarsystem_id', 'to_solarsystem_id')
            ->get(['from_solarsystem_id', 'to_solarsystem_id'])
            ->groupBy('from_solarsystem_id')
            ->map(static fn ($group) => $group->pluck('to_solarsystem_id')->toArray())
            ->all());
    }

    /**
     * Find the fastest path while excluding specific systems
     */
    private function findFastestPath(int $start, int $end, array $connections, array $ignoredSystems): array
    {
        if ($ignoredSystems === []) {
            return $this->findShortestPath($start, $end, $connections);
        }

        // Remove ignored systems from connections (they can't be used as intermediate nodes)
        $filteredConnections = $this->filterConnectionsExcludingSystems($connections, $ignoredSystems);

        return $this->findShortestPath($start, $end, $filteredConnections);
    }

    /**
     * Remove ignored systems from connection graph
     */
    private function filterConnectionsExcludingSystems(array $connections, array $ignoredSystems): array
    {
        if ($ignoredSystems === []) {
            return $connections;
        }

        // Convert to hash map for O(1) lookups instead of O(n) in_array calls
        $ignoredSystemsMap = array_flip($ignoredSystems);
        $filtered = [];

        foreach ($connections as $fromSystem => $targets) {
            // Skip if the source system is ignored
            if (isset($ignoredSystemsMap[$fromSystem])) {
                continue;
            }

            // Filter out ignored target systems using isset() instead of in_array()
            $filteredTargets = array_filter($targets, fn ($target): bool => ! isset($ignoredSystemsMap[$target]));

            if ($filteredTargets !== []) {
                $filtered[$fromSystem] = array_values($filteredTargets);
            }
        }

        return $filtered;
    }

    /**
     * Enrich route path with solarsystem data
     */
    private function enrichRouteWithSolarsystemData(array $path): array
    {
        if ($path === []) {
            return [];
        }

        $solarsystems = Solarsystem::query()
            ->with([
                'sovereignty' => ['alliance', 'corporation', 'faction'],
                'wormholeSystem.effect',
                'constellation',
                'region',
            ])
            ->whereIn('id', $path)
            ->get()
            ->keyBy('id');

        return collect($path)
            ->map(fn (int $id) => $solarsystems->get($id)?->toResource(SolarsystemResource::class))
            ->filter()
            ->values()
            ->toArray();
    }

    /**
     * Find distances to multiple target systems efficiently using a single BFS
     */
    private function findDistancesToMultipleTargets(int $start, array $targets, array $connections, array $ignoredSystems): array
    {
        if ($targets === []) {
            return [];
        }

        // Convert targets to hash map for O(1) lookups
        $targetMap = array_flip($targets);
        $distances = [];
        $visited = [];
        $queue = new SplQueue;

        // Start BFS from the origin
        $queue->enqueue(['id' => $start, 'distance' => 0]);
        $visited[$start] = true;

        // If start is a target, record it
        if (isset($targetMap[$start])) {
            $distances[$start] = 0;
        }

        while (! $queue->isEmpty()) {
            $current = $queue->dequeue();
            $currentId = $current['id'];
            $currentDistance = $current['distance'];

            // Early termination if we found all targets
            if (count($distances) === count($targets)) {
                break;
            }

            $neighbors = $connections[$currentId] ?? [];

            foreach ($neighbors as $neighbor) {
                // Skip if already visited or is an ignored system
                if (isset($visited[$neighbor])) {
                    continue;
                }
                if (in_array($neighbor, $ignoredSystems, true)) {
                    continue;
                }
                $visited[$neighbor] = true;
                $newDistance = $currentDistance + 1;

                // If this neighbor is a target, record its distance
                if (isset($targetMap[$neighbor])) {
                    $distances[$neighbor] = $newDistance;
                }

                $queue->enqueue(['id' => $neighbor, 'distance' => $newDistance]);
            }
        }

        // For unreachable targets, set distance to PHP_INT_MAX
        foreach ($targets as $target) {
            if (! isset($distances[$target])) {
                $distances[$target] = PHP_INT_MAX;
            }
        }

        return $distances;
    }

    private function findShortestPath(int $start, int $end, array $connections): array
    {
        if ($start === $end) {
            return [$start];
        }

        if ($this->isConnectedToSolarsystem($start, $end, $connections)) {
            return [$start, $end];
        }

        $queue = new SplPriorityQueue;
        $visited = [];
        $distances = [$start => 0];
        $bestPathLength = PHP_INT_MAX;

        $queue->insert(new RouteNode($start, 0, [$start]), 0);

        while (! $queue->isEmpty()) {
            $current = $queue->extract();

            if (isset($visited[$current->solarsystemId])) {
                continue;
            }

            // Early termination: if current distance is already >= best found path, skip
            if ($current->distance >= $bestPathLength) {
                continue;
            }

            $visited[$current->solarsystemId] = true;

            if ($current->solarsystemId === $end) {
                return $current->path;
            }

            $neighbors = $connections[$current->solarsystemId] ?? [];

            foreach ($neighbors as $neighbor) {
                if (isset($visited[$neighbor])) {
                    continue;
                }

                $newDistance = $current->distance + 1;

                // Skip if this path is already longer than our best found path
                if ($newDistance >= $bestPathLength) {
                    continue;
                }

                if (! isset($distances[$neighbor]) || $newDistance < $distances[$neighbor]) {
                    $distances[$neighbor] = $newDistance;

                    $newPath = [...$current->path, $neighbor];
                    $heuristic = $this->calculateHeuristic($neighbor, $end);

                    $queue->insert(
                        new RouteNode($neighbor, $newDistance, $newPath, $heuristic),
                        -($newDistance + $heuristic)
                    );
                }
            }
        }

        return [];
    }

    private function isConnectedToSolarsystem(int $from, int $to, array $connections): bool
    {
        return isset($connections[$from]) && in_array($to, $connections[$from], true);
    }

    private function calculateHeuristic(int $from, int $to): float
    {
        // Simple heuristic - could be enhanced with actual distance calculation
        return $from === $to ? 0 : 1;
    }

    private function prepareConnections(RouteOptions $options): array
    {
        $connections = $this->connections;

        if ($options->map instanceof Map) {
            $connections = $this->mergeMapConnections($options->map, $connections, $options->allowEol, $options->massStatus);
        }

        if ($options->allowEveScout) {
            return $this->mergeEveScoutConnections($connections, $options->allowEol);
        }

        return $connections;
    }

    private function mergeMapConnections(Map $map, array $connections, bool $allowEol, MassStatus $massStatus): array
    {
        $mapConnections = $this->getMapConnections($map, $allowEol, $massStatus);

        foreach ($mapConnections as $from => $to) {
            if (isset($connections[$from])) {
                // Use array_push with spread operator for better performance than array_merge
                array_push($connections[$from], ...$to);
            } else {
                $connections[$from] = $to;
            }
        }

        return $connections;
    }

    private function getMapConnections(Map $map, bool $allow_eol, MassStatus $minimum_mass): array
    {
        return MapConnection::query()
            ->join('map_solarsystems as from', 'map_connections.from_map_solarsystem_id', '=', 'from.id')
            ->join('map_solarsystems as to', 'map_connections.to_map_solarsystem_id', '=', 'to.id')
            ->where('map_connections.map_id', $map->id)
            ->when(! $allow_eol, fn (Builder $query) => $query->where('lifetime', LifetimeStatus::Healthy))
            ->when($minimum_mass, fn (Builder $query) => $query->tap(new ConnectionSatisfiesMass($minimum_mass)))
            ->select('from.solarsystem_id as from_solarsystem_id', 'to.solarsystem_id as to_solarsystem_id')
            ->get()
            ->map(static function (MapConnection $connection): array {
                /** @var MapConnection|object{from_solarsystem_id: int, to_solarsystem_id: int} $connection */
                return [
                    ['from_solarsystem_id' => $connection->from_solarsystem_id, 'to_solarsystem_id' => $connection->to_solarsystem_id],
                    ['from_solarsystem_id' => $connection->to_solarsystem_id, 'to_solarsystem_id' => $connection->from_solarsystem_id],
                ];
            })
            ->flatten(1)
            ->groupBy('from_solarsystem_id')
            ->map(static fn ($group) => $group->pluck('to_solarsystem_id')->toArray())
            ->all();
    }

    private function mergeEveScoutConnections(array $connections, bool $allowEol = true): array
    {
        $eveScoutService = app(EveScoutService::class);
        $eveScoutConnections = $eveScoutService->getConnectionsForRouting($allowEol);

        foreach ($eveScoutConnections as $from => $to) {
            if (isset($connections[$from])) {
                // Use array_push with spread operator for better performance than array_merge
                array_push($connections[$from], ...$to);
            } else {
                $connections[$from] = $to;
            }
        }

        return $connections;
    }
}
