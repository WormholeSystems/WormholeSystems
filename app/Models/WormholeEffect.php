<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Support\Collection;

/**
 * Wormhole effect model representing the effects of a wormhole.
 *
 * @property int $id
 * @property string $name
 * @property Collection $effects
 */
class WormholeEffect extends Model
{
    protected $fillable = [
        'name',
        'effects',
    ];

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

        if (! $effects) {
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
            ->map(fn (array $strengths, string $name) => [
                'name' => $name,
                'strength' => $strengths[$class_index] ?? null,
                'type' => 'Buff',
            ])
            ->merge(
                collect($debuffs)
                    ->map(fn (array $strengths, string $name) => [
                        'name' => $name,
                        'strength' => $strengths[$class_index] ?? null,
                        'type' => 'Debuff',
                    ])
            )
            ->values()
            ->all();
    }
}
