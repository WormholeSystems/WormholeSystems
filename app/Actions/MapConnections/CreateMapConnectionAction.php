<?php

namespace App\Actions\MapConnections;

use App\Events\MapConnections\MapConnectionCreatedEvent;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;

class CreateMapConnectionAction
{
    public function handle(array $data): MapConnection
    {
        $map_id = MapSolarsystem::query()
            ->where('id', $data['from_map_solarsystem_id'])
            ->value('map_id');

        $map_connection = MapConnection::create([
            ...$data,
            'map_id' => $map_id,
        ]);

        broadcast(new MapConnectionCreatedEvent($map_id))
            ->toOthers();

        return $map_connection;
    }
}
