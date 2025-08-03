<?php

namespace App\Actions\MapConnections;

use App\Actions\MapSolarsystem\DeleteMapSolarsystemAction;
use App\Events\MapConnections\MapConnectionDeletedEvent;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use Throwable;

readonly class DeleteMapConnectionAction
{
    public function __construct(private DeleteMapSolarsystemAction $deleteMapSolarsystemAction) {}

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

        broadcast(new MapConnectionDeletedEvent($map->id))
            ->toOthers();
    }

    /**
     * @throws Throwable
     */
    private function checkAndRemoveMapSolarsystem(MapSolarsystem $mapSolarsystem): void
    {
        if ($mapSolarsystem->pinned) {
            return;
        }

        if ($mapSolarsystem->connectionsTo()->count() === 0 && $mapSolarsystem->connectionsFrom()->count() === 0) {
            $this->deleteMapSolarsystemAction->handle($mapSolarsystem);
        }
    }
}
