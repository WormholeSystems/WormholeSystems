<?php

namespace App\Actions\MapSelection;

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
                ->delete();
        });
    }
}
