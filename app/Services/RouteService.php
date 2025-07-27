<?php

namespace App\Services;

use App\Enums\MassStatus;
use App\Http\Resources\SolarsystemResource;
use App\Models\Map;
use App\Models\MapConnection;
use App\Models\Solarsystem;
use Illuminate\Database\Eloquent\Builder;
use NicolasKion\SDE\Models\SolarsystemConnection;
use SplPriorityQueue;

class RouteService
{
    private readonly array $connections;

    public const string MAP_CACHE_PATTERN = 'map_%d'; // Keep public for MapRoutesBecameOutdated listener

    public function __construct()
    {
        $this->connections = $this->getConnections();
    }

    private function getConnections(): array
    {
        return SolarsystemConnection::query()
            ->whereDoesntHaveRelation('fromSolarsystem', 'name', '=', 'Zarzakh')
            ->select('from_solarsystem_id', 'to_solarsystem_id')
            ->get(['from_solarsystem_id', 'to_solarsystem_id'])
            ->groupBy('from_solarsystem_id')
            ->map(static fn ($group) => $group->pluck('to_solarsystem_id')->toArray())
            ->all();
    }

    /**
     * Find the fastest route with session-based system exclusions
     */
    public function findRoute(int $fromId, int $toId, RouteOptions $options = new RouteOptions): array
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

        $filtered = [];

        foreach ($connections as $fromSystem => $targets) {
            // Skip if the source system is ignored
            if (in_array($fromSystem, $ignoredSystems)) {
                continue;
            }

            // Filter out ignored target systems
            $filteredTargets = array_filter($targets, fn ($target): bool => ! in_array($target, $ignoredSystems));

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

    private function findShortestPath(int $start, int $end, array $connections): array
    {
        if ($start === $end) {
            return [$start];
        }

        $queue = new SplPriorityQueue;
        $visited = [];
        $distances = [$start => 0];
        $previous = [];

        $queue->insert(new RouteNode($start, 0, [$start]), 0);

        while (! $queue->isEmpty()) {
            $current = $queue->extract();

            if (isset($visited[$current->solarsystemId])) {
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

                if (! isset($distances[$neighbor]) || $newDistance < $distances[$neighbor]) {
                    $distances[$neighbor] = $newDistance;
                    $previous[$neighbor] = $current->solarsystemId;

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

    private function calculateHeuristic(int $from, int $to): float
    {
        // Simple heuristic - could be enhanced with actual distance calculation
        return $from === $to ? 0 : 1;
    }

    private function prepareConnections(RouteOptions $options): array
    {
        $connections = $this->connections;

        if ($options->map instanceof Map) {
            $connections = $this->mergeMapConnections($options->map, $connections, $options->allowEol, $options->allowCrit);
        }

        if ($options->allowEveScout) {
            return $this->mergeEveScoutConnections($connections, $options->allowEol);
        }

        return $connections;
    }

    private function mergeMapConnections(Map $map, array $connections, bool $allowEol, bool $allowCrit): array
    {
        $mapConnections = $this->getMapConnections($map, $allowEol, $allowCrit);

        foreach ($mapConnections as $from => $to) {
            $connections[$from] = isset($connections[$from]) ? array_merge($connections[$from], $to) : $to;
        }

        return $connections;
    }

    private function getMapConnections(Map $map, bool $allowEol, bool $allowCrit): array
    {
        return MapConnection::query()
            ->join('map_solarsystems as from', 'map_connections.from_map_solarsystem_id', '=', 'from.id')
            ->join('map_solarsystems as to', 'map_connections.to_map_solarsystem_id', '=', 'to.id')
            ->where('map_connections.map_id', $map->id)
            ->when(! $allowEol, fn (Builder $query) => $query->where('is_eol', false))
            ->when(! $allowCrit, fn (Builder $query) => $query->where('mass_status', '!=', MassStatus::Critical))
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
            $connections[$from] = isset($connections[$from]) ? array_merge($connections[$from], $to) : $to;
        }

        return $connections;
    }
}
