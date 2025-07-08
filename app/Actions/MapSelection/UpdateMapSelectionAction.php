<?php

namespace App\Actions\MapSelection;

use App\Events\MapSolarsystems\MapSolarsystemsUpdatedEvent;
use App\Models\Map;
use App\Models\MapSolarsystem;
use Illuminate\Support\Facades\DB;
use Throwable;

class UpdateMapSelectionAction
{
    /**
     * @throws Throwable
     */
    public function handle(array $data): void
    {
        DB::transaction(function () use ($data): void {

            $collection = collect($data['map_solarsystems']);
            $collection
                ->each(function (array $item): void {
                    MapSolarsystem::where('id', $item['id'])
                        ->update(['position_x' => $item['position_x'], 'position_y' => $item['position_y']]);
                });

            Map::query()
                ->whereHas('mapSolarsystems', function ($query) use ($data): void {
                    $query->whereIn('id', collect($data['map_solarsystems'])->pluck('id'));
                })
                ->each(fn (Map $map) => broadcast(new MapSolarsystemsUpdatedEvent($map->id))->toOthers());
        });
    }
}
