<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;

/**
 * @property int $id
 * @property int $effect_id
 * @property-read WormholeEffect $effect
 * @property-read Solarsystem $solarsystem
 * @property-read Collection<WormholeStatic> $wormholeStatics
 */
class WormholeSystem extends Model
{
    protected $fillable = [
        'id',
        'effect_id',
    ];

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
        return $this->belongsTo(Solarsystem::class);
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
}
