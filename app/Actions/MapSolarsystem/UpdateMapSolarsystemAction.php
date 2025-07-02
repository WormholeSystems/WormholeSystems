<?php

namespace App\Actions\MapSolarsystem;

use App\Models\MapSolarsystem;

class UpdateMapSolarsystemAction
{
    public function handle(MapSolarsystem $mapSolarsystem, array $data): MapSolarsystem
    {
        return tap($mapSolarsystem)->update($data);
    }
}
