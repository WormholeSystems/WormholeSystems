<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * The static wormholes of a system.
 *
 * @property-read WormholeSystem $wormholeSystem
 * @property-read Wormhole $wormhole
 */
class WormholeStatic extends Model
{
    protected $fillable = [
        'wormhole_id',
        'wormhole_system_id'
    ];

    /**
     * The system that this static wormhole belongs to.
     *
     * @return BelongsTo<WormholeSystem, $this>
     */
    public function wormholeSystem(): BelongsTo
    {
        return $this->belongsTo(WormholeSystem::class);
    }

    /**
     * The wormhole that this static wormhole represents.
     *
     * @return BelongsTo<Wormhole, $this>
     */
    public function wormhole(): BelongsTo
    {
        return $this->belongsTo(Wormhole::class);
    }
}
