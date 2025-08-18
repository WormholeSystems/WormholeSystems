<?php

declare(strict_types=1);

namespace App\Actions\MapSelection;

use App\Events\MapSolarsystems\MapSolarsystemsUpdatedEvent;
use App\Models\Map;
use App\Models\MapSolarsystem;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Support\Facades\DB;
use Throwable;

final class UpdateMapSelectionAction
{
    /**
     * @throws Throwable
     */
    public function handle(array $data): void
    {
        DB::transaction(function () use ($data): void {

            $collection = collect($data['map_solarsystems'])->keyBy('id');

            $models = MapSolarsystem::query()
                ->whereIn('id', $collection->pluck('id'))
                ->get();

            $models->each(function (MapSolarsystem $mapSolarsystem) use ($collection): void {
                $values = $collection->get($mapSolarsystem->id, []);
                $mapSolarsystem->update([
                    'position_x' => $values['position_x'] ?? $mapSolarsystem->position_x,
                    'position_y' => $values['position_y'] ?? $mapSolarsystem->position_y,
                ]);
            });

            Map::query()
                ->whereHas('mapSolarsystems', fn (Builder $query) => $query->whereIn('id', $models->pluck('id')))
                ->each(fn (Map $map) => broadcast(new MapSolarsystemsUpdatedEvent($map->id))->toOthers());
        });
    }
}
