<?php

declare(strict_types=1);

namespace App\Actions\MapConnections;

use App\Models\Map;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use App\Support\Broadcasting\MapBroadcaster;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class CleanStaleMapConnectionsAction
{
    public function __construct(private MapBroadcaster $mapBroadcaster) {}

    /**
     * Remove stale (long-critical) connections from the map and cascade away any systems that are
     * left unreachable from an anchor (a pinned system, or the home system as a fallback).
     *
     * @throws Throwable
     */
    public function handle(Map $map): int
    {
        return DB::transaction(function () use ($map): int {
            $stale_connections = $map->mapConnections()->isStale()->get();

            if ($stale_connections->isEmpty()) {
                return 0;
            }

            $stale_ids = $stale_connections->pluck('id')->all();

            /** @var Collection<int, MapSolarsystem> $systems */
            $systems = $map->mapSolarsystems()->get(['id', 'solarsystem_id', 'pinned']);

            /** @var Collection<int, MapConnection> $surviving_connections */
            $surviving_connections = $map->mapConnections()
                ->whereNotIn('id', $stale_ids)
                ->get(['id', 'from_map_solarsystem_id', 'to_map_solarsystem_id']);

            $map->mapConnections()->whereIn('id', $stale_ids)->delete();

            $anchor_ids = $this->resolveAnchorIds($map, $systems);

            $removed_system_ids = [];

            if ($anchor_ids !== []) {
                $reachable = $this->reachableFrom($anchor_ids, $surviving_connections);

                $removed_systems = $systems
                    ->reject(fn (MapSolarsystem $system): bool => $system->pinned || in_array($system->id, $reachable, true))
                    ->each(fn (MapSolarsystem $system) => $system->delete());

                $removed_system_ids = $removed_systems->pluck('id')->all();
            }

            // Surviving connections touching a cascade-removed system are cascade-deleted
            // by the database, so they are included in the removal payload.
            $cascaded_connection_ids = $surviving_connections
                ->filter(fn (MapConnection $connection): bool => in_array($connection->from_map_solarsystem_id, $removed_system_ids, true)
                    || in_array($connection->to_map_solarsystem_id, $removed_system_ids, true))
                ->pluck('id')
                ->all();

            $this->mapBroadcaster->connectionsRemoved(
                $map->id,
                array_values(array_unique([...$stale_ids, ...$cascaded_connection_ids])),
                $removed_system_ids,
            );

            return $stale_connections->count();
        });
    }

    /**
     * The systems that anchor the map and are never cleaned up: every pinned system plus the
     * home system (which is always protected, even when other systems are pinned).
     *
     * @param  Collection<int, MapSolarsystem>  $systems
     * @return array<int, int>
     */
    private function resolveAnchorIds(Map $map, Collection $systems): array
    {
        $anchors = $systems->where('pinned', true)->pluck('id')->all();

        $home = $systems->firstWhere('solarsystem_id', $map->home_solarsystem_id);

        if ($home instanceof MapSolarsystem) {
            $anchors[] = $home->id;
        }

        return array_values(array_unique($anchors));
    }

    /**
     * Breadth-first search over the (bidirectional) connection graph, returning every system id
     * reachable from any of the given roots.
     *
     * @param  array<int, int>  $root_ids
     * @param  Collection<int, MapConnection>  $connections
     * @return array<int, int>
     */
    private function reachableFrom(array $root_ids, Collection $connections): array
    {
        $adjacency = [];

        foreach ($connections as $connection) {
            $adjacency[$connection->from_map_solarsystem_id][] = $connection->to_map_solarsystem_id;
            $adjacency[$connection->to_map_solarsystem_id][] = $connection->from_map_solarsystem_id;
        }

        $reachable = [];
        $queue = $root_ids;

        while ($queue !== []) {
            $current = array_shift($queue);

            if (isset($reachable[$current])) {
                continue;
            }

            $reachable[$current] = true;

            foreach ($adjacency[$current] ?? [] as $neighbor) {
                if (! isset($reachable[$neighbor])) {
                    $queue[] = $neighbor;
                }
            }
        }

        return array_keys($reachable);
    }
}
