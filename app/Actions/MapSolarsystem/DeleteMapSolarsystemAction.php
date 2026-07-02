<?php

declare(strict_types=1);

namespace App\Actions\MapSolarsystem;

use App\Models\MapSolarsystem;
use App\Support\Broadcasting\MapBroadcaster;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class DeleteMapSolarsystemAction
{
    public function __construct(private MapBroadcaster $mapBroadcaster) {}

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
            $connection_ids = $mapSolarsystem->mapConnections()->get(['id'])->pluck('id')->all();

            $deleted = (bool) $mapSolarsystem->delete();

            $this->mapBroadcaster->systemsRemoved($map_id, [$mapSolarsystem->id], $connection_ids);

            return $deleted;
        });
    }
}
