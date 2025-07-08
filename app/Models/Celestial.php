<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use NicolasKion\SDE\ClassResolver;

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
class Celestial extends Model
{
    public $incrementing = false;

    /**
     * @return BelongsTo<Region,$this>
     */
    public function region(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::region());
    }

    /**
     * @return BelongsTo<Constellation,$this>
     */
    public function constellation(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::constellation());
    }

    /**
     * @return BelongsTo<Type,$this>
     */
    public function type(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::type());
    }

    /**
     * @return BelongsTo<Group,$this>
     */
    public function group(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::group());
    }

    /**
     * @return BelongsTo<Celestial,$this>
     */
    public function parent(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::celestial());
    }

    /**
     * @return HasMany<Station,$this>
     */
    public function stations(): HasMany
    {
        return $this->hasMany(ClassResolver::station(), 'parent_id');
    }
}
