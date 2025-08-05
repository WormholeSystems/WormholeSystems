<?php

namespace App\Models;

use App\Enums\KillmailFilter;
use App\Enums\MassStatus;
use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * Class MapUserSetting
 *
 * @property int $id
 * @property int $user_id
 * @property int $map_id
 * @property bool $tracking_allowed
 * @property bool $is_tracking
 * @property bool $route_allow_eol
 * @property bool $route_use_evescout
 * @property string|MassStatus $route_allow_mass_status
 * @property string|KillmailFilter $killmail_filter
 * @property CarbonImmutable|string $created_at
 * @property CarbonImmutable|string $updated_at
 */
class MapUserSetting extends Model
{
    protected function casts(): array
    {
        return [
            'created_at' => 'immutable_datetime',
            'updated_at' => 'immutable_datetime',
            'tracking_allowed' => 'boolean',
            'is_tracking' => 'boolean',
            'route_allow_eol' => 'boolean',
            'route_allow_mass_status' => MassStatus::class,
            'killmail_filter' => KillmailFilter::class,
            'route_use_evescout' => 'boolean',
        ];
    }

    /**
     * The map that this setting belongs to.
     *
     * @return BelongsTo<Map, $this>
     */
    public function map(): BelongsTo
    {
        return $this->belongsTo(Map::class);
    }

    /**
     * The user that this setting belongs to.
     *
     * @return BelongsTo<User, $this>
     */
    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class);
    }
}
