<?php

namespace App\Actions\MapConnections;

use App\Events\MapConnections\MapConnectionUpdatedEvent;
use App\Models\MapConnection;

class UpdateMapConnectionAction
{
    public function handle(MapConnection $mapConnection, array $data): MapConnection
    {
        $connection = tap($mapConnection)->update($data);

        broadcast(new MapConnectionUpdatedEvent($connection->map_id))
            ->toOthers();

        return $connection;
    }
}
