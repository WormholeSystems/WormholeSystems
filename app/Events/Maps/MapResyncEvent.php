<?php

declare(strict_types=1);

namespace App\Events\Maps;

use App\Events\Concerns\BroadcastsToMap;
use Illuminate\Broadcasting\InteractsWithSockets;
use Illuminate\Contracts\Broadcasting\ShouldBroadcastNow;
use Illuminate\Contracts\Events\ShouldDispatchAfterCommit;
use Illuminate\Foundation\Events\Dispatchable;

/**
 * Contentless ping telling clients the change was too large to broadcast as
 * entity payloads and they should refetch the map state.
 */
final class MapResyncEvent implements ShouldBroadcastNow, ShouldDispatchAfterCommit
{
    use BroadcastsToMap, Dispatchable, InteractsWithSockets;

    public function __construct(public readonly int $map_id) {}

    /**
     * @return array{}
     */
    public function broadcastWith(): array
    {
        return [];
    }
}
