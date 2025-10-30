<?php

declare(strict_types=1);

namespace App\Models;

use App\Enums\KillmailFilter;
use App\Enums\MassStatus;
use App\Enums\RoutePreference;
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
 * @property string|RoutePreference $route_preference
 * @property int $security_penalty
 * @property string|KillmailFilter $killmail_filter
 * @property CarbonImmutable|string|null $introduction_confirmed_at
 * @property bool $prompt_for_signature_enabled
 * @property array|null $layout_breakpoints
 * @property CarbonImmutable|string $created_at
 * @property CarbonImmutable|string $updated_at
 */
final class MapUserSetting extends Model
{
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

    protected function casts(): array
    {
        return [
            'created_at' => 'immutable_datetime',
            'updated_at' => 'immutable_datetime',
            'tracking_allowed' => 'boolean',
            'is_tracking' => 'boolean',
            'route_allow_eol' => 'boolean',
            'route_allow_mass_status' => MassStatus::class,
            'route_preference' => RoutePreference::class,
            'security_penalty' => 'integer',
            'killmail_filter' => KillmailFilter::class,
            'route_use_evescout' => 'boolean',
            'introduction_confirmed_at' => 'immutable_datetime',
            'prompt_for_signature_enabled' => 'boolean',
            'layout_breakpoints' => 'array',
        ];
    }
}
