<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\Solarsystem;
use App\Models\WormholeStatic;
use App\Utilities\CCPRounding;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

use function collect;

/**
 * @mixin Solarsystem
 */
final class SolarsystemResource extends JsonResource
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
            'region_id' => $this->region_id,
            'constellation_id' => $this->constellation_id,
            'class' => $this->wormholeSystem?->class,
            'security' => CCPRounding::roundSecurity($this->security),
            'type' => $this->type,
            'region' => [
                'id' => $this->region->id,
                'name' => $this->region->name,
            ],
            'constellation' => [
                'id' => $this->constellation->id,
                'name' => $this->constellation->name,
            ],
            'sovereignty' => $this->sovereignty?->toResource(SovereigntyResource::class),
            'statics' => $this->wormholeSystem?->wormholeStatics?->map(
                static fn (WormholeStatic $static): array => [
                    'id' => $static->wormhole->id,
                    'leads_to' => $static->wormhole->leads_to,
                ]
            )->values(),
            'effect' => $this->getWormholeEffects(),
            'connection_type' => $this->whenHas('connection_type'),
        ];
    }

    public function getWormholeEffects(): ?array
    {
        $effects = $this->wormholeSystem?->effect?->effects;

        if (! $effects) {
            return null;
        }

        ['Buffs' => $buffs, 'Debuffs' => $debuffs] = $effects;

        $strength = $this->wormholeSystem->class - 1;

        $buffs = collect($buffs)
            ->map(static fn (array $strengths, string $name): array => [
                'name' => $name,
                'strength' => $strengths[$strength] ?? null,
                'type' => 'Buff',
            ])
            ->merge(
                collect($debuffs)
                    ->map(static fn (array $strengths, string $name): array => [
                        'name' => $name,
                        'strength' => $strengths[$strength] ?? null,
                        'type' => 'Debuff',
                    ])
            )
            ->values()
            ->all();

        return [
            'id' => $this->wormholeSystem->effect->id,
            'name' => $this->wormholeSystem->effect->name,
            'effects' => $buffs,
        ];
    }
}
