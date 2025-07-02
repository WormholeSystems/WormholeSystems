<?php

namespace App\Actions\MapConnections;

use App\Models\MapConnection;
use App\Models\MapSolarsystem;

class CreateMapConnectionAction
{
    public function handle(MapSolarsystem $from, MapSolarsystem $to, ?int $wormhole_id = null): MapConnection
    {
        return MapConnection::create([
            'map_id' => $from->map_id,
            'from_map_solarsystem_id' => $from->id,
            'to_map_solarsystem_id' => $to->id,
            'wormhole_id' => $wormhole_id,
            'connected_at' => now(),
        ]);
    }
}
