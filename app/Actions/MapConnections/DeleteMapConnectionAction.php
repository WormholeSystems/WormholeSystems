<?php

namespace App\Actions\MapConnections;

use App\Events\MapConnections\MapConnectionDeletedEvent;
use App\Models\MapConnection;

class DeleteMapConnectionAction
{
    public function handle(MapConnection $mapConnection): void
    {
        $map = $mapConnection->map;

        $mapConnection->delete();

        broadcast(new MapConnectionDeletedEvent($map->id))
            ->toOthers();
    }
}
