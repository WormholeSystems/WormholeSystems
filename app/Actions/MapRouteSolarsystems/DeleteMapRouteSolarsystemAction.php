<?php

declare(strict_types=1);

namespace App\Actions\MapRouteSolarsystems;

use App\Actions\MapRouteSolarsystems\Concerns\BroadcastsRouteSolarsystemChanges;
use App\Models\MapRouteSolarsystem;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class DeleteMapRouteSolarsystemAction
{
    use BroadcastsRouteSolarsystemChanges;

    /**
     * @throws Throwable
     */
    public function handle(MapRouteSolarsystem $mapRouteSolarsystem): bool
    {
        return DB::transaction(function () use ($mapRouteSolarsystem): bool {
            $map_id = $mapRouteSolarsystem->map_id;
            $user_id = $mapRouteSolarsystem->user_id;

            $mapRouteSolarsystem->delete();

            $this->broadcastRouteSolarsystemChange($map_id, $user_id);

            return true;
        });
    }
}
