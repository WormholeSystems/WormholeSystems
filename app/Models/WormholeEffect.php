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
}
