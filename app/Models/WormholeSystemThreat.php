<?php

declare(strict_types=1);

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * Produced by the daily killmail analysis and replaced wholesale on each run.
 *
 * @property int $id
 * @property int $wormhole_system_id
 * @property int $entity_id
 * @property string $entity_type
 * @property string $name
 * @property int $kills
 * @property-read WormholeSystem $wormholeSystem
 */
final class WormholeSystemThreat extends Model
{
    public $timestamps = false;

    /**
     * @return BelongsTo<WormholeSystem, $this>
     */
    public function wormholeSystem(): BelongsTo
    {
        return $this->belongsTo(WormholeSystem::class);
    }
}
