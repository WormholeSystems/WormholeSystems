<?php

declare(strict_types=1);

namespace App\Actions\MapSolarsystem;

use App\Events\MapConnections\MapConnectionsDeletedEvent;
use App\Events\MapSolarsystems\MapSolarsystemDeletedEvent;
use App\Models\MapSolarsystem;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class DeleteMapSolarsystemAction
{
    /**
     * Remove a system from the map. The placement row is hard-deleted so its connections and
     * signatures cascade away; the persistent details (occupier, status, notes) survive.
     *
     * @throws Throwable
     */
    public function handle(MapSolarsystem $mapSolarsystem): bool
    {
        return DB::transaction(function () use ($mapSolarsystem): bool {
            $map_id = $mapSolarsystem->map_id;

            $deleted = (bool) $mapSolarsystem->delete();

            broadcast(new MapSolarsystemDeletedEvent($map_id))->toOthers();
            broadcast(new MapConnectionsDeletedEvent($map_id))->toOthers();

            return $deleted;
        });
    }
}
