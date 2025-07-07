<?php

namespace App\Actions\MapSolarsystem;

use App\Events\MapSolarsystems\MapSolarsystemCreatedEvent;
use App\Models\Map;
use App\Models\MapSolarsystem;

class StoreMapSolarsystemAction
{
    public function handle(Map $map, array $data): MapSolarsystem
    {
        $map_solarsystem = $map->mapSolarsystems()->updateOrCreate([
            'solarsystem_id' => $data['solarsystem_id'],
        ], [
            'position_x' => $data['position_x'] ?? null,
            'position_y' => $data['position_y'] ?? null,
        ]);

        broadcast(new MapSolarsystemCreatedEvent($map->id))
            ->toOthers();

        return $map_solarsystem;
    }
}
