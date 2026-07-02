<?php

declare(strict_types=1);

namespace App\Events\MapSolarsystems;

use App\Events\Concerns\BroadcastsToMap;
use App\Http\Resources\MapSolarsystemResource;
use App\Models\MapSolarsystem;
use Illuminate\Broadcasting\InteractsWithSockets;
use Illuminate\Contracts\Broadcasting\ShouldBroadcastNow;
use Illuminate\Contracts\Events\ShouldDispatchAfterCommit;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Foundation\Events\Dispatchable;

final class MapSolarsystemsUpsertedEvent implements ShouldBroadcastNow, ShouldDispatchAfterCommit
{
    use BroadcastsToMap, Dispatchable, InteractsWithSockets;

    /**
     * The serialized systems, resolved eagerly so eager-loaded counts and relations
     * survive the broadcast queue round-trip.
     *
     * @var array<int, array<string, mixed>>
     */
    public readonly array $map_solarsystems;

    /**
     * @param  Collection<int, MapSolarsystem>  $map_solarsystems
     */
    public function __construct(public readonly int $map_id, Collection $map_solarsystems)
    {
        $this->map_solarsystems = $map_solarsystems->toResourceCollection(MapSolarsystemResource::class)->resolve();
    }

    /**
     * @return array{map_solarsystems: array<int, array<string, mixed>>}
     */
    public function broadcastWith(): array
    {
        return [
            'map_solarsystems' => $this->map_solarsystems,
        ];
    }
}
