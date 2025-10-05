<?php

declare(strict_types=1);

namespace App\Services;

use App\DTO\ClosestSystemResult;
use App\Http\Resources\SolarsystemResource;
use App\Models\Map;
use App\Models\Solarsystem;
use App\Services\Routing\ConnectionRepository;
use App\Services\Routing\PathfindingService;
use App\Services\Routing\RouteCostCalculator;
use Closure;

final readonly class RouteService
{
    public const string MAP_CACHE_PATTERN = 'map_%d'; // Keep public for MapRoutesBecameOutdated listener

    private PathfindingService $pathfinder;

    private ConnectionRepository $connectionRepo;

    private RouteCostCalculator $costCalculator;

    public function __construct()
    {
        $this->connectionRepo = new ConnectionRepository;
        $solarsystems = $this->connectionRepo->getSolarsystems();
        $this->costCalculator = new RouteCostCalculator($solarsystems);
        $this->pathfinder = new PathfindingService($this->costCalculator);
    }

    /**
     * Find the fastest route with session-based system exclusions
     */
    public function findRoute(int $fromId, int $toId, RouteOptions $options): array
    {
        $connections = $this->prepareConnections($options);
        $ignoredSystems = $options->ignoredSystems ?? [];

        // Apply ignored systems filter if needed
        if ($ignoredSystems !== []) {
            $connections = $this->connectionRepo->filterConnectionsExcludingSystems($connections, $ignoredSystems);
        }

        // Find the shortest path
        $path = $this->pathfinder->findShortestPath($fromId, $toId, $connections, $options);

        if ($path === []) {
            return [];
        }

        // Enrich with solarsystem data
        return $this->enrichRouteWithSolarsystemData($path);
    }

    /**
     * Find the closest systems matching a given condition
     *
     * @param  int  $fromId  Starting system ID
     * @param  RouteOptions  $options  Routing options
     * @param  Closure  $condition  Closure to filter target systems
     * @param  int  $limit  Maximum number of results
     * @return array Array of ClosestSystemResult DTOs converted to arrays
     */
    public function findClosestSystems(
        int $fromId,
        RouteOptions $options,
        Closure $condition,
        int $limit = 15
    ): array {
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

        // Find distances to all target systems (returns RouteDistance DTOs)
        $distances = $this->pathfinder->findDistancesToMultipleTargets(
            $fromId,
            $targetSystems,
            $connections,
            $ignoredSystems,
            $options
        );

        // Sort by cost and limit results (client will sort by preference)
        $sortedDistances = collect($distances)
            ->sortBy('cost')
            ->take($limit);

        // Enrich with solarsystem data
        $solarsystemIds = $sortedDistances->pluck('targetSystemId')->toArray();
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

        // Convert to ClosestSystemResult DTOs and then to arrays
        return $sortedDistances
            ->map(function ($distance) use ($solarsystems): ?\App\DTO\ClosestSystemResult {
                $solarsystem = $solarsystems->get($distance->targetSystemId);
                if (! $solarsystem) {
                    return null;
                }

                return new ClosestSystemResult(
                    solarsystem: $solarsystem,
                    cost: $distance->cost,
                    jumps: $distance->jumps
                );
            })
            ->filter()
            ->map(fn (ClosestSystemResult $result): array => $result->toArray())
            ->values()
            ->toArray();
    }

    /**
     * Prepare connection graph based on route options
     */
    private function prepareConnections(RouteOptions $options): array
    {
        $connections = $this->connectionRepo->getBaseConnections();

        // Merge map-specific connections
        if ($options->map instanceof Map) {
            $mapConnections = $this->connectionRepo->getMapConnections(
                $options->map,
                $options->allowEol,
                $options->massStatus
            );
            $connections = $this->connectionRepo->mergeConnections($connections, $mapConnections);
        }

        // Merge EVE Scout connections
        if ($options->allowEveScout) {
            $eveScoutConnections = $this->connectionRepo->getEveScoutConnections($options->allowEol);
            $connections = $this->connectionRepo->mergeConnections($connections, $eveScoutConnections);
        }

        return $connections;
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
}
