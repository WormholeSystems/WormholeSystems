<?php

declare(strict_types=1);

namespace App\Models;

use App\Traits\HasSlug;
use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use NicolasKion\SDE\ClassResolver;

/**
 * Constellation model representing a constellation in the game.
 *
 * @property int $id
 * @property string $name
 * @property int $region_id
 * @property string $type
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Region $region
 * @property-read Collection<int,Solarsystem> $solarsystems
 * @property-read Collection<int,Station> $stations
 * @property-read Collection<int,Celestial> $celestials
 */
class Constellation extends Model
{
    use HasSlug;

    public $incrementing = false;

    /**
     * @return BelongsTo<Region,$this>
     */
    public function region(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::region());
    }

    /**
     * @return HasMany<Solarsystem,$this>
     */
    public function solarsystems(): HasMany
    {
        return $this->hasMany(ClassResolver::solarsystem(), 'constellation_id');
    }

    /**
     * @return HasMany<Station,$this>
     */
    public function stations(): HasMany
    {
        return $this->hasMany(ClassResolver::station(), 'constellation_id');
    }

    /**
     * @return HasMany<Celestial,$this>
     */
    public function celestials(): HasMany
    {
        return $this->hasMany(ClassResolver::celestial(), 'constellation_id');
    }
}
