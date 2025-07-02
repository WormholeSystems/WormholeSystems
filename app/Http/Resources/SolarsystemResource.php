<?php

namespace App\Http\Resources;

use App\Models\Solarsystem;
use App\Utilities\CCPRounding;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Throwable;

/**
 * @mixin Solarsystem
 */
class SolarsystemResource extends JsonResource
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
            'constellation' => $this->constellation->toResource(ConstellationResource::class),
            'region' => $this->region->toResource(RegionResource::class),
            'security' => CCPRounding::roundSecurity($this->security),
            'class' => $this->wormholeSystem?->class,
            'effect' => $this->wormholeSystem?->effect?->name,
            'effects' => $this->wormholeSystem?->effect?->effects->map(fn (array $effect): string => $effect[$this->wormholeSystem->class - 1]),
        ];
    }
}
