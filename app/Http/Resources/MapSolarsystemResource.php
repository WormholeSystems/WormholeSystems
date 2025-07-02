<?php

namespace App\Http\Resources;

use App\Models\MapSolarsystem;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin MapSolarsystem
 */
class MapSolarsystemResource extends JsonResource
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
            'map_id' => $this->map_id,
            'name' => $this->solarsystem->name,
            'solarsystem_id' => $this->solarsystem_id,
            'alias' => $this->alias,
            'occupier_alias' => $this->occupier_alias,
            'position' => $this->position_x ? [
                'x' => $this->position_x,
                'y' => $this->position_y,
            ] : null,
            'status' => $this->status,
            'pinned' => $this->pinned,
            'class' => $this->wormholeSystem?->class,
            'effect' => $this->wormholeSystem?->effect?->name ?? null,
            'effects' => $this->wormholeSystem?->effect?->effects->map(fn (array $effect): string => $effect[$this->wormholeSystem->class - 1]),
            'map_connections' => $this->connections?->toResourceCollection(MapConnectionResource::class),
            'solarsystem' => $this->solarsystem->toResource(SolarsystemResource::class),
        ];
    }
}
