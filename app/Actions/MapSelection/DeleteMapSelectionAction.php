<?php

declare(strict_types=1);

namespace App\Actions\MapSelection;

use App\Events\MapConnections\MapConnectionsDeletedEvent;
use App\Events\MapSolarsystems\MapSolarsystemsDeletedEvent;
use App\Models\MapSolarsystem;
use Illuminate\Support\Facades\DB;
use Throwable;

final class DeleteMapSelectionAction
{
    /**
     * Remove the selected systems from the map. Placement rows are hard-deleted so their
     * connections and signatures cascade away; the persistent details survive.
     *
     * @param  int[]  $map_solarsystem_ids
     *
     * @throws Throwable
     */
    public function handle(array $map_solarsystem_ids): void
    {
        DB::transaction(function () use ($map_solarsystem_ids): void {
            // A selection always belongs to a single map.
            $map_id = MapSolarsystem::query()->whereIn('id', $map_solarsystem_ids)->value('map_id');

            if ($map_id === null) {
                return;
            }

            MapSolarsystem::query()->whereIn('id', $map_solarsystem_ids)->delete();

            broadcast(new MapSolarsystemsDeletedEvent($map_id))->toOthers();
            broadcast(new MapConnectionsDeletedEvent($map_id))->toOthers();
        });
    }
}
