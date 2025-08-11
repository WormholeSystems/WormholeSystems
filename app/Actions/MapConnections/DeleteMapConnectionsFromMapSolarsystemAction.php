<?php

declare(strict_types=1);

namespace App\Actions\MapConnections;

use App\Events\MapConnections\MapConnectionsDeletedEvent;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;

final class DeleteMapConnectionsFromMapSolarsystemAction
{
    /**
     * Handle the deletion of map connections from a solarsystem.
     */
    public function handle(MapSolarsystem $mapSolarsystem): bool
    {
        MapConnection::query()
            ->where('map_connections.from_map_solarsystem_id', $mapSolarsystem->id)
            ->orWhere('map_connections.to_map_solarsystem_id', $mapSolarsystem->id)
            ->delete();

        // Broadcast the event after deletion
        broadcast(new MapConnectionsDeletedEvent($mapSolarsystem->map_id))
            ->toOthers();

        return true;
    }
}
