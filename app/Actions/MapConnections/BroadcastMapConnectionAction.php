<?php

declare(strict_types=1);

namespace App\Actions\MapConnections;

use App\Models\MapConnection;
use App\Support\Broadcasting\MapBroadcaster;

/**
 * Re-broadcasts a single connection with the signature and jump-summary payload
 * every MapConnectionResource serialization expects.
 */
final readonly class BroadcastMapConnectionAction
{
    public function __construct(private MapBroadcaster $mapBroadcaster) {}

    public function handle(MapConnection $connection): void
    {
        $this->mapBroadcaster->connectionsUpserted($connection->map_id, MapConnection::query()
            ->whereKey($connection->id)
            ->with('signatures.signatureType', 'signatures.wormhole')
            ->withJumpSummary()
            ->get());
    }
}
