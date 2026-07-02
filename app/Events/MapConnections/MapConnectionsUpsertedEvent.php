<?php

declare(strict_types=1);

namespace App\Events\MapConnections;

use App\Events\Concerns\BroadcastsToMap;
use App\Http\Resources\MapConnectionResource;
use App\Models\MapConnection;
use Illuminate\Broadcasting\InteractsWithSockets;
use Illuminate\Contracts\Broadcasting\ShouldBroadcastNow;
use Illuminate\Contracts\Events\ShouldDispatchAfterCommit;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Foundation\Events\Dispatchable;

final class MapConnectionsUpsertedEvent implements ShouldBroadcastNow, ShouldDispatchAfterCommit
{
    use BroadcastsToMap, Dispatchable, InteractsWithSockets;

    /**
     * The serialized connections, resolved eagerly so eager-loaded relations
     * survive the broadcast queue round-trip.
     *
     * @var array<int, array<string, mixed>>
     */
    public readonly array $map_connections;

    /**
     * @param  Collection<int, MapConnection>  $map_connections
     */
    public function __construct(public readonly int $map_id, Collection $map_connections)
    {
        $this->map_connections = $map_connections->toResourceCollection(MapConnectionResource::class)->resolve();
    }

    /**
     * @return array{map_connections: array<int, array<string, mixed>>}
     */
    public function broadcastWith(): array
    {
        return [
            'map_connections' => $this->map_connections,
        ];
    }
}
