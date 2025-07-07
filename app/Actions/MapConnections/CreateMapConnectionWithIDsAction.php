<?php

namespace App\Actions\MapConnections;

use App\Events\MapConnections\MapConnectionCreatedEvent;
use App\Models\Map;

readonly class CreateMapConnectionWithIDsAction
{
    public function __construct(private CreateMapConnectionAction $connectMapSolarsystems) {}

    public function handle(Map $map, int $from_solarsystem_id, int $to_solarsystem_id, ?int $wormhole_id = null): void
    {
        $from_solarsystem = $map->mapSolarsystems()
            ->where('solarsystem_id', $from_solarsystem_id)
            ->firstOrFail();

        $to_solarsystem = $map->mapSolarsystems()
            ->where('solarsystem_id', $to_solarsystem_id)
            ->firstOrFail();

        $this->connectMapSolarsystems->handle([
            'from_map_solarsystem_id' => $from_solarsystem->id,
            'to_map_solarsystem_id' => $to_solarsystem->id,
            'wormhole_id' => $wormhole_id,
        ]);

        broadcast(new MapConnectionCreatedEvent($map->id))
            ->toOthers();
    }
}
