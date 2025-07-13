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
            $mapRouteSolarsystem->delete();

            broadcast(new MapRouteSolarsystemsUpdatedEvent($mapRouteSolarsystem->id))->toOthers();

            return true;
        });
    }
}
