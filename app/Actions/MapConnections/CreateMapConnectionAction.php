<?php

declare(strict_types=1);

namespace App\Actions\MapConnections;

use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use App\Support\Broadcasting\MapBroadcaster;

final class CreateMapConnectionAction
{
    public function __construct(private readonly MapBroadcaster $mapBroadcaster) {}

    public function handle(array $data): MapConnection
    {
        $map_id = MapSolarsystem::query()
            ->where('id', $data['from_map_solarsystem_id'])
            ->value('map_id');

        $map_connection = MapConnection::create([
            ...$data,
            'map_id' => $map_id,
        ]);

        $this->mapBroadcaster->connectionsUpserted($map_id, MapConnection::query()
            ->whereKey($map_connection->id)
            ->with('signatures.signatureType', 'signatures.wormhole')
            ->get());

        return $map_connection;
    }
}
