<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\CharacterStatus;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Throwable;

/**
 * @mixin CharacterStatus
 */
final class CharacterStatusResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     *
     * @throws Throwable
     */
    public function toArray(Request $request): array
    {
        return [
            'solarsystem_id' => $this->solarsystem_id,
            'station_id' => $this->station_id,
            'structure_id' => $this->structure_id,
            'ship_name' => $this->ship_name,
            'ship_type' => $this->shipType->toResource(TypeResource::class),
            'solarsystem' => $this->solarsystem?->toResource(SolarsystemResource::class),
            'is_online' => $this->is_online,
            'last_online_at' => $this->last_online_at,
        ];
    }
}
