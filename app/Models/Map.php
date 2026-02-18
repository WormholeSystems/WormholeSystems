<?php

declare(strict_types=1);

namespace App\Models;

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
 * @property string|null $share_token
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Collection<int,MapSolarsystem> $mapSolarsystems
 * @property-read Collection<int,MapConnection> $mapConnections
 * @property-read Collection<int,MapAccess> $mapAccessors
 * @property-read Collection<int,MapRouteSolarsystem> $mapRouteSolarsystems
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
        ];
    }
}
