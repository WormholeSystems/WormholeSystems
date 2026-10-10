<?php

declare(strict_types=1);

namespace App\Actions\MapRouteSolarsystems;

use App\Actions\MapRouteSolarsystems\Concerns\BroadcastsRouteSolarsystemChanges;
use App\Models\MapRouteSolarsystem;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class UpdateMapRouteSolarsystemAction
{
    use BroadcastsRouteSolarsystemChanges;

    /**
     * @throws Throwable
     */
    public function handle(MapRouteSolarsystem $mapRouteSolarsystem, array $data): MapRouteSolarsystem
    {
        return DB::transaction(function () use ($mapRouteSolarsystem, $data): MapRouteSolarsystem {
            $mapRouteSolarsystem->update($data);

            $this->broadcastRouteSolarsystemChange($mapRouteSolarsystem->map_id, $mapRouteSolarsystem->user_id);

            return $mapRouteSolarsystem;
        });
    }
}
