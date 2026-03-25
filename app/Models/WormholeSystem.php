<?php

declare(strict_types=1);

namespace App\Models;

use App\Enums\ThreatLevel;
use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;

/**
 * WormholeSystem model representing a wormhole system in the game.
 *
 * @property int $id
 * @property int $effect_id
 * @property-read int $class
 * @property ThreatLevel $threat_level
 * @property array|null $threat_data
 * @property CarbonImmutable|null $threat_analyzed_at
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read WormholeEffect $effect
 * @property-read Solarsystem $solarsystem
 * @property-read Collection<int,WormholeStatic> $wormholeStatics
 */
final class WormholeSystem extends Model
{
    /**
     * The effect of the wormhole system.
     *
     * @return BelongsTo<WormholeEffect, $this>
     */
    public function effect(): BelongsTo
    {
        return $this->belongsTo(WormholeEffect::class);
    }

    /**
     * The solar system that this wormhole system belongs to.
     *
     * @return BelongsTo<Solarsystem, $this>
     */
    public function solarsystem(): BelongsTo
    {
        return $this->belongsTo(Solarsystem::class, 'id');
    }

    /**
     * The static wormholes that connect to this wormhole system.
     *
     * @return HasMany<WormholeStatic, $this>
     */
    public function wormholeStatics(): HasMany
    {
        return $this->hasMany(WormholeStatic::class, 'wormhole_system_id');
    }

    /**
     * @return array<string, string>
     */
    protected function casts(): array
    {
        return [
            'threat_level' => ThreatLevel::class,
            'threat_data' => 'array',
            'threat_analyzed_at' => 'immutable_datetime',
            'created_at' => 'immutable_datetime',
            'updated_at' => 'immutable_datetime',
        ];
    }
}
