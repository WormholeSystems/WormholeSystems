<?php

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\HasMany;

/**
 * Wormhole effect model representing the effects of a wormhole.
 *
 * @property int $id
 * @property string $name
 * @property Collection $effects
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Collection<int,WormholeSystem> $wormholeSystems
 */
class WormholeEffect extends Model
{
    protected function casts(): array
    {
        return [
            'effects' => 'collection',
        ];
    }

    /**
     * @return HasMany<WormholeSystem,$this>
     */
    public function wormholeSystems(): HasMany
    {
        return $this->hasMany(WormholeSystem::class, 'effect_id');
    }

    public function getEffectArray(string $class): ?array
    {
        $effects = $this->effects;

        if ($effects->isEmpty()) {
            return null;
        }

        ['Buffs' => $buffs, 'Debuffs' => $debuffs] = $effects;

        $class_index = match ((int) $class) {
            1 => 0,
            2 => 1,
            3 => 2,
            4 => 3,
            5 => 4,
            6 => 5,
            13 => 6,
            default => match (true) {
                $class >= 14 && $class <= 18 => 1,
                default => $class,
            }

        };

        return collect($buffs)
            ->map(fn (array $strengths, string $name): array => [
                'name' => $name,
                'strength' => $strengths[$class_index] ?? null,
                'type' => 'Buff',
            ])
            ->merge(
                collect($debuffs)
                    ->map(fn (array $strengths, string $name): array => [
                        'name' => $name,
                        'strength' => $strengths[$class_index] ?? null,
                        'type' => 'Debuff',
                    ])
            )
            ->values()
            ->all();
    }
}
