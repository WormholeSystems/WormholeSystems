<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\Map;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Throwable;

/**
 * @mixin Map
 */
final class MapResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     * @return array<string, mixed>
     *
     * @throws Throwable
     */
    public function toArray(Request $request): array
    {
        return [
            'id' => $this->id,
            'name' => $this->name,
            'slug' => $this->slug,
            'map_solarsystems' => $this->mapSolarsystems->toResourceCollection(MapSolarsystemResource::class),
            'map_connections' => $this->mapConnections->toResourceCollection(MapConnectionResource::class),
            'owner' => [
                'id' => $this->mapOwner->id,
                'character_name' => $this->mapOwner->accessible->name,
            ],
        ];
    }
}
