<?php

namespace App\Actions\MapSolarsystem;

use App\Actions\MapConnections\DeleteMapConnectionsFromMapSolarsystemAction;
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
        return DB::transaction(function () use ($mapSolarsystem) {
            $mapSolarsystem->update(['position_x' => null, 'position_y' => null, 'alias' => null]);

            return $this->action->handle($mapSolarsystem);
        });
    }
}
