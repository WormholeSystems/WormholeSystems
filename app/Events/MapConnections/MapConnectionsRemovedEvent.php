<?php

declare(strict_types=1);

namespace App\Events\MapConnections;

use App\Events\Concerns\BroadcastsToMap;
use Illuminate\Broadcasting\InteractsWithSockets;
use Illuminate\Contracts\Broadcasting\ShouldBroadcastNow;
use Illuminate\Contracts\Events\ShouldDispatchAfterCommit;
use Illuminate\Foundation\Events\Dispatchable;

final class MapConnectionsRemovedEvent implements ShouldBroadcastNow, ShouldDispatchAfterCommit
{
    use BroadcastsToMap, Dispatchable, InteractsWithSockets;

    /**
     * @param  int[]  $map_connection_ids
     * @param  int[]  $map_solarsystem_ids  Systems cascade-removed together with the connections.
     */
    public function __construct(
        public readonly int $map_id,
        public readonly array $map_connection_ids,
        public readonly array $map_solarsystem_ids,
    ) {}

    /**
     * @return array{map_connection_ids: int[], map_solarsystem_ids: int[]}
     */
    public function broadcastWith(): array
    {
        return [
            'map_connection_ids' => $this->map_connection_ids,
            'map_solarsystem_ids' => $this->map_solarsystem_ids,
        ];
    }
}
