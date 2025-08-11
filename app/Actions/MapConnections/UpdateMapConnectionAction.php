<?php

declare(strict_types=1);

namespace App\Actions\MapConnections;

use App\Events\MapConnections\MapConnectionUpdatedEvent;
use App\Models\MapConnection;

final class UpdateMapConnectionAction
{
    public function handle(MapConnection $mapConnection, array $data): MapConnection
    {
        $connection = tap($mapConnection)->update($data);

        broadcast(new MapConnectionUpdatedEvent($connection->map_id))
            ->toOthers();

        return $connection;
    }
}
