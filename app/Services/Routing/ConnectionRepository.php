<?php

declare(strict_types=1);

namespace App\Services\Routing;

use App\Enums\LifetimeStatus;
use App\Enums\MassStatus;
use App\Models\Map;
use App\Models\MapConnection;
use App\Models\Solarsystem;
use App\Scopes\ConnectionSatisfiesMass;
use App\Services\EveScoutService;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Support\Facades\Cache;
use NicolasKion\SDE\Models\SolarsystemConnection;

final readonly class ConnectionRepository
{
    private const string BASE_CONNECTIONS_CACHE_KEY = 'route_service_base_connections';

    private const string SOLARSYSTEMS_CACHE_KEY = 'route_service_solarsystems';

    private const int CACHE_TTL = 3600; // 1 hour

    /**
     * Get all base EVE Online connections (cached)
     */
    public function getBaseConnections(): array
    {
        return Cache::remember(
            self::BASE_CONNECTIONS_CACHE_KEY,
            self::CACHE_TTL,
            static fn (): array => SolarsystemConnection::query()
                ->whereDoesntHaveRelation('fromSolarsystem', 'name', '=', 'Zarzakh')
                ->select('from_solarsystem_id', 'to_solarsystem_id')
                ->get(['from_solarsystem_id', 'to_solarsystem_id'])
                ->groupBy('from_solarsystem_id')
                ->map(static fn ($group) => $group->pluck('to_solarsystem_id')->toArray())
                ->all()
        );
    }

    /**
     * Get solarsystem data (cached)
     */
    public function getSolarsystems(): array
    {
        return Cache::remember(
            self::SOLARSYSTEMS_CACHE_KEY,
            self::CACHE_TTL,
            static fn (): array => Solarsystem::query()
                ->select('id', 'security')
                ->get()
                ->keyBy('id')
                ->toArray()
        );
    }

    /**
     * Get map-specific connections
     */
    public function getMapConnections(Map $map, bool $allowEol, MassStatus $minimumMass): array
    {
        return MapConnection::query()
            ->join('map_solarsystems as from', 'map_connections.from_map_solarsystem_id', '=', 'from.id')
            ->join('map_solarsystems as to', 'map_connections.to_map_solarsystem_id', '=', 'to.id')
            ->where('map_connections.map_id', $map->id)
            ->when(! $allowEol, fn (Builder $query) => $query->where('lifetime', LifetimeStatus::Healthy))
            ->when($minimumMass, fn (Builder $query) => $query->tap(new ConnectionSatisfiesMass($minimumMass)))
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

    /**
     * Get EVE Scout connections
     */
    public function getEveScoutConnections(bool $allowEol): array
    {
        $eveScoutService = app(EveScoutService::class);

        return $eveScoutService->getConnectionsForRouting($allowEol);
    }

    /**
     * Merge additional connections into the base connection graph
     */
    public function mergeConnections(array $baseConnections, array $additionalConnections): array
    {
        $connections = $baseConnections;

        foreach ($additionalConnections as $from => $to) {
            if (isset($connections[$from])) {
                // Use array_push with spread operator for better performance
                array_push($connections[$from], ...$to);
            } else {
                $connections[$from] = $to;
            }
        }

        return $connections;
    }

    /**
     * Filter connections to exclude specific systems
     */
    public function filterConnectionsExcludingSystems(array $connections, array $ignoredSystems): array
    {
        if ($ignoredSystems === []) {
            return $connections;
        }

        // Convert to hash map for O(1) lookups
        $ignoredSystemsMap = array_flip($ignoredSystems);
        $filtered = [];

        foreach ($connections as $fromSystem => $targets) {
            // Skip if the source system is ignored
            if (isset($ignoredSystemsMap[$fromSystem])) {
                continue;
            }

            // Filter out ignored target systems
            $filteredTargets = array_filter($targets, fn ($target): bool => ! isset($ignoredSystemsMap[$target]));

            if ($filteredTargets !== []) {
                $filtered[$fromSystem] = array_values($filteredTargets);
            }
        }

        return $filtered;
    }

    /**
     * Create a set of wormhole connection keys from connection array
     * Returns an array with keys like "1234-5678" for quick lookup
     */
    public function getWormholeConnectionSet(array $connections): array
    {
        $wormholeSet = [];

        foreach ($connections as $from => $targets) {
            foreach ($targets as $to) {
                // Use consistent key format (smaller ID first)
                $key = $from < $to ? "{$from}-{$to}" : "{$to}-{$from}";
                $wormholeSet[$key] = true;
            }
        }

        return $wormholeSet;
    }
}
