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
            'effects' => $this->wormholeSystem?->effect?->getEffectArray($this->wormholeSystem->class),
        ];
    }

    public function getWormholeEffects(): ?array
    {
        $effects = $this->wormholeSystem?->effect?->effects;

        if (! $effects) {
            return null;
        }

        ['Buffs' => $buffs, 'Debuffs' => $debuffs] = $effects;

        $this->dump($buffs, $debuffs, $effects);

        // Map the effects into single array with 'name', 'strength', and 'type' keys

        return collect($buffs)
            ->map(fn (array $strengths, string $name) => [
                'name' => $name,
                'strength' => $strengths[$this->wormholeSystem->class - 1] ?? null,
                'type' => 'Buff',
            ])
            ->merge(
                collect($debuffs)
                    ->map(fn (array $strengths, string $name) => [
                        'name' => $name,
                        'strength' => $strengths[$this->wormholeSystem->class - 1] ?? null,
                        'type' => 'Debuff',
                    ])
            )
            ->values()
            ->all();
    }
}
