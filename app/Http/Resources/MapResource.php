<?php

namespace App\Http\Resources;

use App\Models\Map;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin Map
 */
class MapResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     * @return array<string, mixed>
     */
    public function toArray(Request $request): array
    {
        return [
            'id' => $this->id,
            'name' => $this->name,
            'slug' => $this->slug,
            'map_solarsystems' => $this->whenLoaded('mapSolarsystems', fn () => $this->mapSolarsystems->toResourceCollection(MapSolarsystemResource::class)),
            'map_connections' => $this->whenLoaded('mapConnections', fn () => $this->mapConnections->toResourceCollection(MapConnectionResource::class)),
            'map_solarsystems_count' => $this->whenCounted('mapSolarsystems', fn () => $this->map_solarsystems_count),
        ];
    }
}
