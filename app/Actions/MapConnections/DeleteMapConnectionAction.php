<?php

declare(strict_types=1);

namespace App\Actions\MapConnections;

use App\Actions\MapSolarsystem\DeleteMapSolarsystemAction;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use App\Support\Broadcasting\MapBroadcaster;
use Throwable;

final readonly class DeleteMapConnectionAction
{
    public function __construct(
        private DeleteMapSolarsystemAction $deleteMapSolarsystemAction,
        private MapBroadcaster $mapBroadcaster,
    ) {}

    /**
     * @throws Throwable
     */
    public function handle(MapConnection $mapConnection, bool $remove_map_solarsystem = false): void
    {
        $map = $mapConnection->map;

        $from_map_solarsystem = $mapConnection->fromMapSolarsystem;
        $to_map_solarsystem = $mapConnection->toMapSolarsystem;

        $mapConnection->delete();

        if ($remove_map_solarsystem) {
            $this->checkAndRemoveMapSolarsystem($from_map_solarsystem);
            $this->checkAndRemoveMapSolarsystem($to_map_solarsystem);
        }

        // Deleting the placement flips $exists on the very instances passed above, which
        // is how the cascade-removed systems are detected here.
        $removed_system_ids = collect([$from_map_solarsystem, $to_map_solarsystem])
            ->reject(fn (MapSolarsystem $map_solarsystem): bool => $map_solarsystem->exists)
            ->pluck('id')
            ->all();

        $this->mapBroadcaster->connectionsRemoved($map->id, [$mapConnection->id], $removed_system_ids);
    }

    /**
     * @throws Throwable
     */
    private function checkAndRemoveMapSolarsystem(MapSolarsystem $mapSolarsystem): void
    {
        if ($mapSolarsystem->pinned) {
            return;
        }

        if (! $mapSolarsystem->mapConnections()->exists()) {
            $this->deleteMapSolarsystemAction->handle($mapSolarsystem);
        }
    }
}
