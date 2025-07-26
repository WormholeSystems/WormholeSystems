<?php

namespace App\Services;

use App\Enums\MassStatus;
use App\Http\Resources\SolarsystemResource;
use App\Models\Map;
use App\Models\MapConnection;
use App\Models\Solarsystem;
use App\Services\EveScoutService;
use App\Utilities\DomainLogic;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Http\Resources\Json\JsonResource;
use Illuminate\Support\Facades\Cache;
use JMGQ\AStar\AStar;
use NicolasKion\SDE\Models\SolarsystemConnection;
use Throwable;

/**
 * @property array<int, int[]> $connections
 */
class RouteService
{
    private readonly array $connections;

    private const int CACHE_EXPIRATION_SECONDS = 60 * 60 * 24;

    public const string MAP_CACHE_PATTERN = 'map_%d';

    public const string MAP_ROUTE_CACHE_PATTERN = 'route_%d_%d_map_%d_eol_%d_crit_%d_scouts_%d';

    public const string ROUTE_CACHE_PATTERN = 'route_%d_%d_eol_%d_crit_%d_scouts_%d';

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
     * @return int[]
     *
     * @throws Throwable
     */
    public function find(int $from_id, int $to_id, ?Map $map = null, ?bool $allow_eol = true, ?bool $allow_crit = false, ?bool $allow_eve_scout = false): array
    {
        if (($cached_route = $this->getCachedRoute($from_id, $to_id, $map, $allow_eol, $allow_crit, $allow_eve_scout)) !== null) {
            return $cached_route;
        }

        $connections = $this->connections;
        if ($map instanceof Map) {
            $connections = $this->mergeMapConnections($map, $connections, $allow_eol, $allow_crit);
        }

        // Add EVE Scout connections if enabled
        if ($allow_eve_scout === true) {
            $connections = $this->mergeEveScoutConnections($connections, $allow_eol);
        }

        $domain_logic = new DomainLogic($connections);
        $star = new AStar($domain_logic);

        $route = $star->run($from_id, $to_id);

        $solarsystems = Solarsystem::query()
            ->with([
                'sovereignty' => [
                    'alliance',
                    'corporation',
                    'faction',
                ],
                'wormholeSystem.effect',
                'constellation',
                'region',
            ])
            ->whereIn('id', array_values($route))->get()->keyBy('id');

        $route = array_map(static fn (int $id): JsonResource => $solarsystems->get($id)->toResource(SolarsystemResource::class), $route);

        $this->setCachedRoute($from_id, $to_id, $route, $map, $allow_eol, $allow_crit, $allow_eve_scout);

        return $route;
    }

    private function mergeMapConnections(Map $map, array $connections, bool $allow_eol, bool $allow_crit): array
    {
        $map_connections = $this->getMapConnections($map, $allow_eol, $allow_crit);

        foreach ($map_connections as $from => $to) {
            $connections[$from] = isset($connections[$from]) ? array_merge($connections[$from], $to) : $to;
        }

        return $connections;
    }

    private function getMapConnections(Map $map, bool $allow_eol, bool $allow_crit): array
    {
        return MapConnection::query()
            ->join('map_solarsystems as from', 'map_connections.from_map_solarsystem_id', '=', 'from.id')
            ->join('map_solarsystems as to', 'map_connections.to_map_solarsystem_id', '=', 'to.id')
            ->where('map_connections.map_id', $map->id)
            ->when(! $allow_eol, fn (Builder $query) => $query->where('is_eol', false))
            ->when(! $allow_crit, fn (Builder $query) => $query->where('mass_status', '!=', MassStatus::Critical))
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

    private function mergeEveScoutConnections(array $connections, bool $allow_eol = true): array
    {
        $eve_scout_service = app(EveScoutService::class);
        $eve_scout_connections = $eve_scout_service->getConnectionsForRouting($allow_eol);

        foreach ($eve_scout_connections as $from => $to) {
            $connections[$from] = isset($connections[$from]) ? array_merge($connections[$from], $to) : $to;
        }

        return $connections;
    }

    private function getCachedRoute(int $from_id, int $to_id, ?Map $map = null, ?bool $allow_eol = true, ?bool $allow_crit = false, ?bool $allow_eve_scout = false): ?array
    {
        if ($map instanceof Map) {
            return $this->getCachedRouteForMap($from_id, $to_id, $map, $allow_eol, $allow_crit, $allow_eve_scout);
        }

        $key = $this->getRouteCacheKey($from_id, $to_id, $allow_eol, $allow_crit, $allow_eve_scout);

        return Cache::get($key);
    }

    private function getCachedRouteForMap(int $from_id, int $to_id, Map $map, ?bool $allow_eol = true, ?bool $allow_crit = false, ?bool $allow_eve_scout = false): ?array
    {
        $key = $this->getMapRouteCacheKey($from_id, $to_id, $map, $allow_eol, $allow_crit, $allow_eve_scout);

        $tag = $this->getMapCacheKey($map);

        return Cache::tags($tag)->get($key);
    }

    private function setCachedRoute(int $from_id, int $to_id, array $route, ?Map $map = null, ?bool $allow_eol = true, ?bool $allow_crit = false, ?bool $allow_eve_scout = false): void
    {
        if ($map instanceof Map) {
            $this->setCachedRouteForMap($from_id, $to_id, $route, $map, $allow_eol, $allow_crit, $allow_eve_scout);

            return;
        }

        $key_1 = $this->getRouteCacheKey($from_id, $to_id, $allow_eol, $allow_crit, $allow_eve_scout);
        $key_2 = $this->getRouteCacheKey($to_id, $from_id, $allow_eol, $allow_crit, $allow_eve_scout);

        Cache::put($key_1, $route, self::CACHE_EXPIRATION_SECONDS);
        Cache::put($key_2, array_reverse($route), self::CACHE_EXPIRATION_SECONDS);
    }

    private function setCachedRouteForMap(int $from_id, int $to_id, array $route, Map $map, ?bool $allow_eol = true, ?bool $allow_crit = false, ?bool $allow_eve_scout = false): void
    {
        $key_1 = $this->getMapRouteCacheKey($from_id, $to_id, $map, $allow_eol, $allow_crit, $allow_eve_scout);
        $key_2 = $this->getMapRouteCacheKey($to_id, $from_id, $map, $allow_eol, $allow_crit, $allow_eve_scout);

        $tag = $this->getMapCacheKey($map);

        Cache::tags($tag)->put($key_1, $route, self::CACHE_EXPIRATION_SECONDS);
        Cache::tags($tag)->put($key_2, array_reverse($route), self::CACHE_EXPIRATION_SECONDS);
    }

    private function getMapCacheKey(Map $map): string
    {
        return sprintf(self::MAP_CACHE_PATTERN, $map->id);
    }

    private function getMapRouteCacheKey(int $from_id, int $to_id, Map $map, ?bool $allow_eol = true, ?bool $allow_crit = false, ?bool $allow_eve_scout = false): string
    {
        return sprintf(self::MAP_ROUTE_CACHE_PATTERN, $from_id, $to_id, $map->id, (int) $allow_eol, (int) $allow_crit, (int) $allow_eve_scout);
    }

    private function getRouteCacheKey(int $from_id, int $to_id, ?bool $allow_eol = true, ?bool $allow_crit = false, ?bool $allow_eve_scout = false): string
    {
        return sprintf(self::ROUTE_CACHE_PATTERN, $from_id, $to_id, (int) $allow_eol, (int) $allow_crit, (int) $allow_eve_scout);
    }
}
