<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\RaidableSkyhook;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin RaidableSkyhook
 */
final class RaidableSkyhookResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     */
    public function toArray(Request $request): array
    {
        return [
            'planet_id' => $this->planet_id,
            'solarsystem_id' => $this->solarsystem_id,
            'planet_name' => $this->planet?->name,
            'planet_type' => $this->resolvePlanetType(),
            'theft_vulnerability_start' => $this->theft_vulnerability_start,
            'theft_vulnerability_end' => $this->theft_vulnerability_end,
        ];
    }

    private function resolvePlanetType(): ?string
    {
        return match ($this->planet?->type_id) {
            null => null,
            2015, 56020 => 'lava',
            12, 56019 => 'ice',
            default => 'other',
        };
    }
}
