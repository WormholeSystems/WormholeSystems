<?php

namespace App\Actions\MapSolarsystem;

use App\Models\MapSolarsystem;

class DeleteMapSolarsystemAction
{
    public function handle(MapSolarsystem $mapSolarsystem): bool
    {
        return $mapSolarsystem->delete();
    }
}
