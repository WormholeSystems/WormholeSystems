<?php

declare(strict_types=1);

namespace App\Events\Maps;

use App\Events\Concerns\BroadcastsToMap;
use App\Http\Resources\MapMetadataResource;
use App\Models\Map;
use Illuminate\Broadcasting\InteractsWithSockets;
use Illuminate\Contracts\Broadcasting\ShouldBroadcastNow;
use Illuminate\Contracts\Events\ShouldDispatchAfterCommit;
use Illuminate\Foundation\Events\Dispatchable;

final class MapMetadataUpdatedEvent implements ShouldBroadcastNow, ShouldDispatchAfterCommit
{
    use BroadcastsToMap, Dispatchable, InteractsWithSockets;

    public readonly int $map_id;

    /**
     * The serialized map metadata (never the systems or connections).
     *
     * @var array<string, mixed>
     */
    public readonly array $map;

    public function __construct(Map $map)
    {
        $this->map_id = $map->id;
        $this->map = $map->toResource(MapMetadataResource::class)->resolve();
    }

    /**
     * @return array{map: array<string, mixed>}
     */
    public function broadcastWith(): array
    {
        return [
            'map' => $this->map,
        ];
    }
}
