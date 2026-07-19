<?php

declare(strict_types=1);

namespace App\Models;

use App\Enums\AliasScheme;
use App\Enums\MapLayout;
use App\Enums\Permission;
use App\Traits\HasSlug;
use Carbon\CarbonImmutable;
use Context;
use Database\Factories\MapFactory;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Database\Eloquent\Relations\HasOne;

use function sprintf;

/**
 * A player map with solar systems and their connections.
 *
 * @property int $id
 * @property string $name
 * @property bool $is_public
 * @property bool $constant_width_enabled
 * @property string|null $share_token
 * @property int|null $home_solarsystem_id
 * @property int|null $rally_solarsystem_id
 * @property string $bookmark_format_wormhole
 * @property string $bookmark_format_kspace
 * @property AliasScheme $bookmark_alias_scheme
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Collection<int,MapSolarsystem> $mapSolarsystems
 * @property-read Collection<int,MapSolarsystemDetails> $mapSolarsystemDetails
 * @property-read Collection<int,MapConnection> $mapConnections
 * @property-read Collection<int,MapAccess> $mapAccessors
 * @property-read Collection<int,MapRouteSolarsystem> $mapRouteSolarsystems
 * @property-read Collection<int,MapIgnoredSolarsystem> $mapIgnoredSolarsystems
 * @property-read Collection<int,MapWebhook> $mapWebhooks
 * @property-read Collection<int,MapWebhookRole> $mapWebhookRoles
 * @property-read Collection<int,MapAlert> $mapAlerts
 * @property-read null|MapUserSetting $mapUserSetting
 * @property-read MapAccess $mapOwner
 */
final class Map extends Model
{
    /** @use HasFactory<MapFactory> */
    use HasFactory, HasSlug;

    public function isPubliclyAccessible(): bool
    {
        return $this->is_public || $this->share_token !== null;
    }

    /**
     * The map solar systems that are part of this map.
     *
     * @return HasMany<MapConnection, $this>
     */
    public function mapConnections(): HasMany
    {
        return $this->hasMany(MapConnection::class, 'map_id');
    }

    /**
     * The connections between solar systems in this map.
     *
     * @return HasMany<MapSolarsystem, $this>
     */
    public function mapSolarsystems(): HasMany
    {
        return $this->hasMany(MapSolarsystem::class, 'map_id');
    }

    /**
     * Persistent per-system intel for this map, including systems not currently placed.
     *
     * @return HasMany<MapSolarsystemDetails, $this>
     */
    public function mapSolarsystemDetails(): HasMany
    {
        return $this->hasMany(MapSolarsystemDetails::class, 'map_id');
    }

    /**
     * The access control entries for this map.
     *
     * @return HasMany<MapAccess, $this>
     */
    public function mapAccessors(): HasMany
    {
        return $this->hasMany(MapAccess::class, 'map_id');
    }

    public function mapOwner(): HasOne
    {
        return $this->hasOne(MapAccess::class, 'map_id')
            ->where('is_owner', true);
    }

    /**
     * The route solar systems for this map.
     *
     * @return HasMany<MapRouteSolarsystem, $this>
     */
    public function mapRouteSolarsystems(): HasMany
    {
        return $this->hasMany(MapRouteSolarsystem::class, 'map_id');
    }

    /**
     * The solar systems that should never be auto-mapped on this map.
     *
     * @return HasMany<MapIgnoredSolarsystem, $this>
     */
    public function mapIgnoredSolarsystems(): HasMany
    {
        return $this->hasMany(MapIgnoredSolarsystem::class, 'map_id');
    }

    /**
     * The Discord webhooks configured for this map.
     *
     * @return HasMany<MapWebhook, $this>
     */
    public function mapWebhooks(): HasMany
    {
        return $this->hasMany(MapWebhook::class, 'map_id');
    }

    /**
     * The reusable Discord roles configured for this map.
     *
     * @return HasMany<MapWebhookRole, $this>
     */
    public function mapWebhookRoles(): HasMany
    {
        return $this->hasMany(MapWebhookRole::class, 'map_id');
    }

    /**
     * The alerts configured for this map.
     *
     * @return HasMany<MapAlert, $this>
     */
    public function mapAlerts(): HasMany
    {
        return $this->hasMany(MapAlert::class, 'map_id');
    }

    public function mapUserSettings(): HasMany
    {
        return $this->hasMany(MapUserSetting::class, 'map_id');
    }

    public function mapUserSetting(): HasOne
    {
        return $this->hasOne(MapUserSetting::class, 'map_id')
            ->where('user_id', auth()->id())
            ->ofMany();
    }

    /**
     * Get the user's permission level for this map.
     */
    public function getUserPermission(User $user): ?Permission
    {
        return Context::remember(sprintf('map_%d_user_%d_permission', $this->id, $user->id), fn () => $this->mapAccessors()
            ->notExpired()
            ->whereIn('accessible_id', $user->getAccessibleIds())
            ->orderByRaw("CASE WHEN permission = 'manager' THEN 1 WHEN permission = 'member' THEN 2 WHEN permission = 'viewer' THEN 3 ELSE 4 END")
            ->first()?->permission
        );
    }

    /**
     * @return array<string, string>
     */
    protected function casts(): array
    {
        return [
            'is_public' => 'boolean',
            'layout' => MapLayout::class,
            'allow_layout_override' => 'boolean',
            'constant_width_enabled' => 'boolean',
            'bookmark_alias_scheme' => AliasScheme::class,
        ];
    }
}
