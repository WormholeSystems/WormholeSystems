<?php

declare(strict_types=1);

namespace App\Models;

use App\Traits\HasSlug;
use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Database\Eloquent\Relations\HasOne;
use NicolasKion\SDE\ClassResolver;

/**
 * Solarsystem model representing a solar system in the game.
 *
 * @property int $id
 * @property string $name
 * @property int $constellation_id
 * @property int $region_id
 * @property float $security
 * @property float $pos_x
 * @property float $pos_y
 * @property float $pos_z
 * @property string $type
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Constellation $constellation
 * @property-read Region $region
 * @property-read Collection<int,Celestial> $celestials
 * @property-read Collection<int,Station> $stations
 * @property-read WormholeSystem|null $wormholeSystem
 * @property-read Sovereignty|null $sovereignty
 * @property-read Collection<int,MapSolarsystem> $mapSolarsystems
 */
class Solarsystem extends Model
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
     * @return BelongsTo<Constellation,$this>
     */
    public function constellation(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::constellation());
    }

    /**
     * @return HasMany<Celestial,$this>
     */
    public function celestials(): HasMany
    {
        return $this->hasMany(ClassResolver::celestial(), 'solarsystem_id');
    }

    /**
     * @return HasMany<Station,$this>
     */
    public function stations(): HasMany
    {
        return $this->hasMany(ClassResolver::station(), 'solarsystem_id');
    }

    /**
     * @return HasOne<WormholeSystem,$this>
     */
    public function wormholeSystem(): HasOne
    {
        return $this->hasOne(WormholeSystem::class, 'id', 'id');
    }

    /**
     * MapSolarsystems related to this solarsystem.
     *
     * @return HasMany<MapSolarsystem,$this>
     */
    public function mapSolarsystems(): HasMany
    {
        return $this->hasMany(MapSolarsystem::class, 'solarsystem_id');
    }

    /**
     * @return HasOne<Sovereignty,$this>
     */
    public function sovereignty(): HasOne
    {
        return $this->hasOne(Sovereignty::class, 'solarsystem_id');
    }
}
