<?php

declare(strict_types=1);

namespace App\Actions\MapRouteSolarsystems;

use App\Events\MapRouteSolarsystemsUpdatedEvent;
use App\Models\MapRouteSolarsystem;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class DeleteMapRouteSolarsystemAction
{
    /**
     * @throws Throwable
     */
    public function handle(MapRouteSolarsystem $mapRouteSolarsystem): bool
    {
        return DB::transaction(function () use ($mapRouteSolarsystem): bool {
            $map_id = $mapRouteSolarsystem->map_id;

            $mapRouteSolarsystem->delete();

            broadcast(new MapRouteSolarsystemsUpdatedEvent($map_id))->toOthers();

            return true;
        });
    }
}
