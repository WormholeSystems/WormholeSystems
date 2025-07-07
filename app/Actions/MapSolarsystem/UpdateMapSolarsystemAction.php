<?php

namespace App\Actions\MapSolarsystem;

use App\Events\MapSolarsystems\MapSolarsystemsUpdatedEvent;
use App\Models\MapSolarsystem;

class UpdateMapSolarsystemAction
{
    public function handle(MapSolarsystem $mapSolarsystem, array $data): MapSolarsystem
    {
        $system = tap($mapSolarsystem)->update($data);

        broadcast(new MapSolarsystemsUpdatedEvent($system->map_id))
            ->toOthers();

        return $system;
    }
}
