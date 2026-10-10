<?php

declare(strict_types=1);

namespace App\Actions\MapRouteSolarsystems;

use App\Actions\MapRouteSolarsystems\Concerns\BroadcastsRouteSolarsystemChanges;
use App\Models\Map;
use App\Models\MapRouteSolarsystem;
use Illuminate\Support\Facades\DB;
use Throwable;

final readonly class CreateMapRouteSolarsystemAction
{
    use BroadcastsRouteSolarsystemChanges;

    /**
     * Add a system to a watchlist, or update its pin when it is already there. A null (or
     * missing) user_id targets the shared watchlist.
     *
     * @param  array{map_id: int, solarsystem_id: int, user_id?: int|null, is_pinned?: bool|null}  $data
     *
     * @throws Throwable
     */
    public function handle(array $data): MapRouteSolarsystem
    {
        return DB::transaction(function () use ($data): MapRouteSolarsystem {
            $user_id = $data['user_id'] ?? null;

            if ($user_id === null) {
                // The unique index cannot catch shared duplicates (NULL user_id), so concurrent
                // shared inserts on the same map are serialized on the map row instead.
                Map::query()->lockForUpdate()->find($data['map_id']);
            }

            $mapRouteSolarsystem = MapRouteSolarsystem::query()->updateOrCreate(
                [
                    'map_id' => $data['map_id'],
                    'user_id' => $user_id,
                    'solarsystem_id' => $data['solarsystem_id'],
                ],
                isset($data['is_pinned']) ? ['is_pinned' => $data['is_pinned']] : [],
            );

            $this->broadcastRouteSolarsystemChange($mapRouteSolarsystem->map_id, $user_id);

            return $mapRouteSolarsystem;
        });
    }
}
