<?php

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * The static wormholes of a system.
 *
 * @property int $id
 * @property int $wormhole_id
 * @property int $wormhole_system_id
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read WormholeSystem $wormholeSystem
 * @property-read Wormhole $wormhole
 */
class WormholeStatic extends Model
{
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
