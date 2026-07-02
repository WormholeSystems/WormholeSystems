<?php

declare(strict_types=1);

namespace App\Events\MapSolarsystems;

use App\Events\Concerns\BroadcastsToMap;
use Illuminate\Broadcasting\InteractsWithSockets;
use Illuminate\Contracts\Broadcasting\ShouldBroadcastNow;
use Illuminate\Contracts\Events\ShouldDispatchAfterCommit;
use Illuminate\Foundation\Events\Dispatchable;

final class MapSolarsystemsRemovedEvent implements ShouldBroadcastNow, ShouldDispatchAfterCommit
{
    use BroadcastsToMap, Dispatchable, InteractsWithSockets;

    /**
     * @param  int[]  $map_solarsystem_ids
     * @param  int[]  $map_connection_ids  Connections cascade-removed with the systems, captured before the delete.
     */
    public function __construct(
        public readonly int $map_id,
        public readonly array $map_solarsystem_ids,
        public readonly array $map_connection_ids,
    ) {}

    /**
     * @return array{map_solarsystem_ids: int[], map_connection_ids: int[]}
     */
    public function broadcastWith(): array
    {
        return [
            'map_solarsystem_ids' => $this->map_solarsystem_ids,
            'map_connection_ids' => $this->map_connection_ids,
        ];
    }
}
