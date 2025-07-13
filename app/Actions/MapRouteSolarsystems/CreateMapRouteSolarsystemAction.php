<?php

declare(strict_types=1);

namespace App\Actions\MapRouteSolarsystems;

use App\Events\MapRouteSolarsystemsUpdatedEvent;
use App\Models\MapRouteSolarsystem;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class CreateMapRouteSolarsystemAction
{
    /**
     * @throws Throwable
     */
    public function handle(array $data): MapRouteSolarsystem
    {
        return DB::transaction(function () use ($data): MapRouteSolarsystem {
            $data = MapRouteSolarsystem::query()->create($data);
            broadcast(new MapRouteSolarsystemsUpdatedEvent($data['map_id']))->toOthers();

            return $data;
        });
    }
}
