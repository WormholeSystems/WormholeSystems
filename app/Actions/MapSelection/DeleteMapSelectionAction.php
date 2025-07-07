<?php

namespace App\Actions\MapSelection;

use App\Events\MapSolarsystems\MapSolarsystemsDeletedEvent;
use App\Models\Map;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use Illuminate\Support\Facades\DB;
use Throwable;

class DeleteMapSelectionAction
{
    /**
     * Delete selected map solarsystems from the database.
     *
     * @param  int[]  $map_solarsystem_ids
     *
     * @throws Throwable
     */
    public function handle(array $map_solarsystem_ids): void
    {
        DB::transaction(function () use ($map_solarsystem_ids) {
            MapSolarsystem::query()->whereIn('id', $map_solarsystem_ids)
                ->update(['position_x' => null, 'position_y' => null, 'alias' => null]);

            MapConnection::query()
                ->whereIn('from_map_solarsystem_id', $map_solarsystem_ids)
                ->orWhereIn('to_map_solarsystem_id', $map_solarsystem_ids)
                ->delete();

            Map::query()
                ->whereHas('mapSolarsystems', function ($query) use ($map_solarsystem_ids) {
                    $query->whereIn('id', $map_solarsystem_ids);
                })
                ->each(fn (Map $map) => broadcast(new MapSolarsystemsDeletedEvent($map->id))->toOthers());
        });
    }
}
