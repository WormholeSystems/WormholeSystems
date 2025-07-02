<?php

namespace App\Actions\MapSolarsystem;

use App\Models\Map;
use App\Models\MapSolarsystem;

class StoreMapSolarsystemAction
{
    public function handle(Map $map, array $data): MapSolarsystem
    {
        return $map->mapSolarsystems()->create($data);
    }
}
