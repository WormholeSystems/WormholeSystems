<?php

namespace App\Actions\MapSolarsystem;

use App\Actions\MapConnections\DeleteMapConnectionsFromMapSolarsystemAction;
use App\Events\MapSolarsystems\MapSolarsystemDeletedEvent;
use App\Models\MapSolarsystem;
use Illuminate\Support\Facades\DB;
use Throwable;

readonly class DeleteMapSolarsystemAction
{
    public function __construct(private DeleteMapConnectionsFromMapSolarsystemAction $action) {}

    /**
     * @throws Throwable
     */
    public function handle(MapSolarsystem $mapSolarsystem): bool
    {
        return DB::transaction(function () use ($mapSolarsystem): bool {
            $mapSolarsystem->update(['position_x' => null, 'position_y' => null, 'alias' => null]);

            broadcast(new MapSolarsystemDeletedEvent($mapSolarsystem->map_id))
                ->toOthers();

            return $this->action->handle($mapSolarsystem);
        });
    }
}
