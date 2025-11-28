<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;

/**
 * Celestial model representing a celestial object in the game.
 *
 * @property int $id
 * @property string $name
 * @property int $solarsystem_id
 * @property int $constellation_id
 * @property int $region_id
 * @property int|null $parent_id
 * @property int $type_id
 * @property int $group_id
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Region $region
 * @property-read Constellation $constellation
 * @property-read Type $type
 * @property-read Group $group
 * @property-read Celestial|null $parent
 * @property-read Collection<int,Station> $stations
 */
final class Celestial extends Model
{
    public $incrementing = false;

    /**
     * @return BelongsTo<Region,$this>
     */
    public function region(): BelongsTo
    {
        return $this->belongsTo(Region::class);
    }

    /**
     * @return BelongsTo<Constellation,$this>
     */
    public function constellation(): BelongsTo
    {
        return $this->belongsTo(Constellation::class);
    }

    /**
     * @return BelongsTo<Type,$this>
     */
    public function type(): BelongsTo
    {
        return $this->belongsTo(Type::class);
    }

    /**
     * @return BelongsTo<Group,$this>
     */
    public function group(): BelongsTo
    {
        return $this->belongsTo(Group::class);
    }

    /**
     * @return BelongsTo<Celestial,$this>
     */
    public function parent(): BelongsTo
    {
        return $this->belongsTo(self::class);
    }

    /**
     * @return HasMany<Station,$this>
     */
    public function stations(): HasMany
    {
        return $this->hasMany(Station::class, 'parent_id');
    }

    /**
     * Child celestials (e.g., moons orbiting a planet).
     *
     * @return HasMany<Celestial,$this>
     */
    public function children(): HasMany
    {
        return $this->hasMany(self::class, 'parent_id');
    }

    /**
     * Moons orbiting this celestial (filtered children with group_id 8).
     *
     * @return HasMany<Celestial,$this>
     */
    public function moons(): HasMany
    {
        return $this->hasMany(self::class, 'parent_id')->where('group_id', 8);
    }
}
