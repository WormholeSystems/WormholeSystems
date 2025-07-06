<?php

namespace App\Actions\MapConnections;

use App\Models\MapConnection;
use App\Models\MapSolarsystem;

class CreateMapConnectionAction
{
    public function handle(array $data): MapConnection
    {
        $map_id = MapSolarsystem::query()
            ->where('id', $data['from_map_solarsystem_id'])
            ->value('map_id');

        return MapConnection::create([
            ...$data,
            'map_id' => $map_id,
        ]);
    }
}
