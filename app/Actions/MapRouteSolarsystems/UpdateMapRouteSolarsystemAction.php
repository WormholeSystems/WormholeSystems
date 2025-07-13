<?php

declare(strict_types=1);

namespace App\Actions\MapRouteSolarsystems;

use App\Events\MapRouteSolarsystemsUpdatedEvent;
use App\Models\MapRouteSolarsystem;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class UpdateMapRouteSolarsystemAction
{
    /**
     * @throws Throwable
     */
    public function handle(MapRouteSolarsystem $mapRouteSolarsystem, array $data): MapRouteSolarsystem
    {
        return DB::transaction(function () use ($mapRouteSolarsystem, $data): MapRouteSolarsystem {
            $mapRouteSolarsystem->update($data);

            broadcast(new MapRouteSolarsystemsUpdatedEvent($mapRouteSolarsystem->map_id))->toOthers();

            return $mapRouteSolarsystem;
        });
    }
}
