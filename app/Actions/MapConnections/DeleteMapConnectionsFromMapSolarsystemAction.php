<?php

namespace App\Actions\MapConnections;

use App\Models\MapConnection;
use App\Models\MapSolarsystem;

class DeleteMapConnectionsFromMapSolarsystemAction
{
    /**
     * Handle the deletion of map connections from a solarsystem.
     */
    public function handle(MapSolarsystem $mapSolarsystem): bool
    {
        return MapConnection::query()
            ->where('map_connections.from_map_solarsystem_id', $mapSolarsystem->id)
            ->orWhere('map_connections.to_map_solarsystem_id', $mapSolarsystem->id)
            ->delete();
    }
}
