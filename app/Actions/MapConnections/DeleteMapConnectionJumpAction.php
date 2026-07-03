<?php

declare(strict_types=1);

namespace App\Actions\MapConnections;

use App\Models\MapConnection;
use App\Models\MapConnectionJump;

/**
 * Removes a logged jump from a connection's mass estimate and pushes the
 * refreshed summary to the map.
 */
final readonly class DeleteMapConnectionJumpAction
{
    public function __construct(private BroadcastMapConnectionAction $broadcastMapConnection) {}

    public function handle(MapConnectionJump $jump): void
    {
        $connection = $jump->mapConnection;

        $jump->delete();

        if ($connection instanceof MapConnection) {
            $this->broadcastMapConnection->handle($connection);
        }
    }
}
