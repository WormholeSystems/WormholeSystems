<?php

declare(strict_types=1);

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * @property int $id
 * @property int $map_alert_id
 * @property int $map_solarsystem_id
 * @property \Carbon\CarbonImmutable|null $delivered_at
 * @property \Carbon\CarbonImmutable $created_at
 * @property \Carbon\CarbonImmutable $updated_at
 */
final class MapAlertDelivery extends Model
{
    /** @return BelongsTo<MapAlert, $this> */
    public function alert(): BelongsTo
    {
        return $this->belongsTo(MapAlert::class, 'map_alert_id');
    }

    /** @return BelongsTo<MapSolarsystem, $this> */
    public function mapSolarsystem(): BelongsTo
    {
        return $this->belongsTo(MapSolarsystem::class);
    }

    /** @return array<string, string> */
    protected function casts(): array
    {
        return ['delivered_at' => 'immutable_datetime'];
    }
}
