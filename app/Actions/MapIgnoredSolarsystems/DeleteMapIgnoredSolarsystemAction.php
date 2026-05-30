<?php

declare(strict_types=1);

namespace App\Actions\MapIgnoredSolarsystems;

use App\Events\MapIgnoredSolarsystemsUpdatedEvent;
use App\Models\MapIgnoredSolarsystem;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class DeleteMapIgnoredSolarsystemAction
{
    /**
     * @throws Throwable
     */
    public function handle(MapIgnoredSolarsystem $mapIgnoredSolarsystem): bool
    {
        return DB::transaction(function () use ($mapIgnoredSolarsystem): bool {
            $map_id = $mapIgnoredSolarsystem->map_id;

            $mapIgnoredSolarsystem->delete();

            broadcast(new MapIgnoredSolarsystemsUpdatedEvent($map_id))->toOthers();

            return true;
        });
    }
}
