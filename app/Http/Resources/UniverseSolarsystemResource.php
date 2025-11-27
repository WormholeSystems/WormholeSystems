<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\Solarsystem;
use App\Utilities\CCPRounding;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin Solarsystem
 */
final class UniverseSolarsystemResource extends JsonResource
{
    /**
     * Normalization factor to convert EVE coordinates to canvas coordinates.
     * EVE coords are ~10^18, we want ~10^4 for reasonable canvas size.
     */
    private const float COORDINATE_SCALE = 1e14;

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
            'region_id' => $this->region_id,
            'constellation_id' => $this->constellation_id,
            'security' => CCPRounding::roundSecurity($this->security),
            'type' => $this->type,
            'position' => [
                'x' => $this->pos_2d_x / self::COORDINATE_SCALE,
                'y' => $this->pos_2d_y / self::COORDINATE_SCALE,
            ],
            'region' => [
                'id' => $this->region->id,
                'name' => $this->region->name,
            ],
            'constellation' => [
                'id' => $this->constellation->id,
                'name' => $this->constellation->name,
            ],
            'class' => $this->wormholeSystem?->class,
            'sovereignty' => $this->sovereignty?->toResource(SovereigntyResource::class),
            'has_stations' => ($this->stations_count ?? 0) > 0,
            'has_belts' => ($this->belts_count ?? 0) > 0,
        ];
    }
}
