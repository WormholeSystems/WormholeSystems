<?php

namespace App\Services;

use App\Models\Map;
use App\Models\MapConnection;
use App\Utilities\DomainLogic;
use Illuminate\Support\Facades\Cache;
use JMGQ\AStar\AStar;
use NicolasKion\SDE\Models\SolarsystemConnection;

/**
 * @property array<int, int[]> $connections
 */
class RouteService
{
    private readonly array $connections;

    private const int CACHE_EXPIRATION_SECONDS = 60 * 60 * 24;

    public const string MAP_CACHE_PATTERN = 'map_%d';

    public const string MAP_ROUTE_CACHE_PATTERN = 'route_%d_%d_map_%d';

    public const string ROUTE_CACHE_PATTERN = 'route_%d_%d';

    public function __construct()
    {
        $this->connections = $this->getConnections();
    }

    private function getConnections(): array
    {
        return SolarsystemConnection::query()
            ->select('from_solarsystem_id', 'to_solarsystem_id')
            ->get(['from_solarsystem_id', 'to_solarsystem_id'])
            ->groupBy('from_solarsystem_id')
            ->map(static fn ($group) => $group->pluck('to_solarsystem_id')->toArray())
            ->all();
    }

    /**
     * @return int[]
     */
    public function find(int $from_id, int $to_id, ?Map $map = null): array
    {
        if (($cached_route = $this->getCachedRoute($from_id, $to_id, $map)) !== null) {
            return $cached_route;
        }

        $connections = $this->connections;
        if ($map instanceof Map) {
            $connections = $this->mergeMapConnections($map, $connections);
        }

        $domain_logic = new DomainLogic($connections);
        $star = new AStar($domain_logic);

        $route = $star->run($from_id, $to_id);

        $this->setCachedRoute($from_id, $to_id, $route, $map);

        return $route;
    }

    private function mergeMapConnections(Map $map, array $connections): array
    {
        $map_connections = $this->getMapConnections($map);

        foreach ($map_connections as $from => $to) {
            $connections[$from] = isset($connections[$from]) ? array_merge($connections[$from], $to) : $to;
        }

        return $connections;
    }

    private function getMapConnections(Map $map): array
    {
        return MapConnection::query()
            ->join('map_solarsystems as from', 'map_connections.from_map_solarsystem_id', '=', 'from.id')
            ->join('map_solarsystems as to', 'map_connections.to_map_solarsystem_id', '=', 'to.id')
            ->where('map_connections.map_id', $map->id)
            ->select('from.solarsystem_id as from_solarsystem_id', 'to.solarsystem_id as to_solarsystem_id')
            ->get()
            ->map(static function (MapConnection $connection): array {
                /** @var MapConnection|object{from_solarsystem_id: int, to_solarsystem_id: int} $connection */
                return [
                    ['from_solarsystem_id' => $connection->from_solarsystem_id,
                        'to_solarsystem_id' => $connection->to_solarsystem_id,
                    ],
                    ['from_solarsystem_id' => $connection->to_solarsystem_id,
                        'to_solarsystem_id' => $connection->from_solarsystem_id,
                    ],
                ];
            }
            )
            ->flatten(1)
            ->groupBy('from_solarsystem_id')
            ->map(static fn ($group) => $group->pluck('to_solarsystem_id')->toArray())->all();
    }

    private function getCachedRoute(int $from_id, int $to_id, ?Map $map = null): ?array
    {
        if ($map instanceof Map) {
            return $this->getCachedRouteForMap($from_id, $to_id, $map);
        }

        $key = $this->getRouteCacheKey($from_id, $to_id);

        return Cache::get($key);
    }

    private function getCachedRouteForMap(int $from_id, int $to_id, Map $map): ?array
    {
        $key = $this->getMapRouteCacheKey($from_id, $to_id, $map);

        $tag = $this->getMapCacheKey($map);

        return Cache::tags($tag)->get($key);
    }

    private function setCachedRoute(int $from_id, int $to_id, array $route, ?Map $map = null): void
    {
        if ($map instanceof Map) {
            $this->setCachedRouteForMap($from_id, $to_id, $route, $map);

            return;
        }

        $key_1 = $this->getRouteCacheKey($from_id, $to_id);
        $key_2 = $this->getRouteCacheKey($to_id, $from_id);

        Cache::put($key_1, $route, self::CACHE_EXPIRATION_SECONDS);
        Cache::put($key_2, array_reverse($route), self::CACHE_EXPIRATION_SECONDS);
    }

    private function setCachedRouteForMap(int $from_id, int $to_id, array $route, Map $map): void
    {
        $key_1 = $this->getMapRouteCacheKey($from_id, $to_id, $map);
        $key_2 = $this->getMapRouteCacheKey($to_id, $from_id, $map);

        $tag = $this->getMapCacheKey($map);

        Cache::tags($tag)->put($key_1, $route, self::CACHE_EXPIRATION_SECONDS);
        Cache::tags($tag)->put($key_2, array_reverse($route), self::CACHE_EXPIRATION_SECONDS);
    }

    private function getMapCacheKey(Map $map): string
    {
        return sprintf(self::MAP_CACHE_PATTERN, $map->id);
    }

    private function getMapRouteCacheKey(int $from_id, int $to_id, Map $map): string
    {
        return sprintf(self::MAP_ROUTE_CACHE_PATTERN, $from_id, $to_id, $map->id);
    }

    private function getRouteCacheKey(int $from_id, int $to_id): string
    {
        return sprintf(self::ROUTE_CACHE_PATTERN, $from_id, $to_id);
    }
}
