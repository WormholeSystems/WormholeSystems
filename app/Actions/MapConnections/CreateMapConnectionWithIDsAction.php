<?php

namespace App\Actions\MapConnections;

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

        $this->connectMapSolarsystems->handle(
            $from_solarsystem,
            $to_solarsystem,
            $wormhole_id
        );
    }
}
