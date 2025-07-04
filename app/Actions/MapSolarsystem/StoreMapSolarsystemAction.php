<?php

namespace App\Actions\MapSolarsystem;

use App\Models\Map;
use App\Models\MapSolarsystem;

class StoreMapSolarsystemAction
{
    public function handle(Map $map, array $data): MapSolarsystem
    {
        return $map->mapSolarsystems()->updateOrCreate([
            'solarsystem_id' => $data['solarsystem_id'],
        ], [
            'position_x' => $data['position_x'] ?? null,
            'position_y' => $data['position_y'] ?? null,
        ]);
    }
}
