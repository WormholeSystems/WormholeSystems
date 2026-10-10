<?php

declare(strict_types=1);

namespace App\Actions\MapRouteSolarsystems\Concerns;

use App\Events\MapRouteSolarsystemsUpdatedEvent;
use App\Events\MapUserRouteSolarsystemsUpdatedEvent;

trait BroadcastsRouteSolarsystemChanges
{
    /**
     * Shared changes go to every map user; personal changes only to the owner's other tabs.
     */
    private function broadcastRouteSolarsystemChange(int $map_id, ?int $user_id): void
    {
        if ($user_id === null) {
            broadcast(new MapRouteSolarsystemsUpdatedEvent($map_id))->toOthers();

            return;
        }

        broadcast(new MapUserRouteSolarsystemsUpdatedEvent($user_id, $map_id))->toOthers();
    }
}
