<?php

declare(strict_types=1);

namespace App\Actions\MapConnections;

use App\Enums\ShipSize;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use App\Models\Wormhole;
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

        /* A known wormhole type dictates the ship size regardless of what the
         * caller passed; the hole's physics are not a matter of preference.
         */
        $wormhole_ship_size = ShipSize::fromJumpMass(
            Wormhole::query()->find($data['wormhole_id'] ?? null)?->maximum_jump_mass
        );
        if ($wormhole_ship_size instanceof ShipSize) {
            $data['ship_size'] = $wormhole_ship_size;
        }

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
