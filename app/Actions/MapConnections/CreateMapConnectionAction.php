<?php

declare(strict_types=1);

namespace App\Actions\MapConnections;

use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use App\Support\Broadcasting\MapBroadcaster;

final readonly class CreateMapConnectionAction
{
    public function __construct(
        private MapBroadcaster $mapBroadcaster,
        private ClaimPendingConnectionJumpsAction $claimPendingConnectionJumpsAction,
    ) {}

    public function handle(array $data): MapConnection
    {
        $map_id = MapSolarsystem::query()
            ->where('id', $data['from_map_solarsystem_id'])
            ->value('map_id');

        $map_connection = MapConnection::create([
            ...$data,
            'map_id' => $map_id,
        ]);

        /* Claim before the broadcast so the first payload other viewers see
         * already carries jumps observed before the connection existed.
         */
        $this->claimPendingConnectionJumpsAction->handle($map_connection);

        $this->mapBroadcaster->connectionsUpserted($map_id, MapConnection::query()
            ->whereKey($map_connection->id)
            ->with('signatures.signatureType', 'signatures.wormhole')
            ->withJumpSummary()
            ->get());

        return $map_connection;
    }
}
