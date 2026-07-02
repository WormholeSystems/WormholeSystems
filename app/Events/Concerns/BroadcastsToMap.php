<?php

declare(strict_types=1);

namespace App\Events\Concerns;

use Illuminate\Broadcasting\Channel;
use Illuminate\Broadcasting\PrivateChannel;

/**
 * Broadcasts the event on the private channel of the map identified by the
 * event's public int $map_id property.
 */
trait BroadcastsToMap
{
    /**
     * Get the channels the event should broadcast on.
     *
     * @return array<int, Channel>
     */
    public function broadcastOn(): array
    {
        return [
            new PrivateChannel(sprintf('Map.%d', $this->map_id)),
        ];
    }
}
