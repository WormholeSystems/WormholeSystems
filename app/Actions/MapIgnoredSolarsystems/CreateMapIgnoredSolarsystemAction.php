<?php

declare(strict_types=1);

namespace App\Actions\MapIgnoredSolarsystems;

use App\Events\MapIgnoredSolarsystemsUpdatedEvent;
use App\Models\MapIgnoredSolarsystem;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class CreateMapIgnoredSolarsystemAction
{
    /**
     * @param  array{map_id: int, solarsystem_id: int}  $data
     *
     * @throws Throwable
     */
    public function handle(array $data): MapIgnoredSolarsystem
    {
        return DB::transaction(function () use ($data): MapIgnoredSolarsystem {
            $mapIgnoredSolarsystem = MapIgnoredSolarsystem::query()->firstOrCreate($data);

            broadcast(new MapIgnoredSolarsystemsUpdatedEvent($mapIgnoredSolarsystem->map_id))->toOthers();

            return $mapIgnoredSolarsystem;
        });
    }
}
