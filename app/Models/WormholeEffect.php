<?php

declare(strict_types=1);

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
final class WormholeEffect extends Model
{
    /**
     * @return HasMany<WormholeSystem,$this>
     */
    public function wormholeSystems(): HasMany
    {
        return $this->hasMany(WormholeSystem::class, 'effect_id');
    }

    protected function casts(): array
    {
        return [
            'effects' => 'collection',
        ];
    }
}
