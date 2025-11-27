<?php

declare(strict_types=1);

namespace App\Models;

use App\Builders\SolarsystemBuilder;
use App\Traits\HasSlug;
use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Attributes\UseEloquentBuilder;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Database\Eloquent\Relations\HasOne;

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
 * @property float|null $pos_2d_x
 * @property float|null $pos_2d_y
 * @property string $type
 * @property bool $has_jove_observatory
 * @property string|null $connection_type
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Constellation $constellation
 * @property-read Region $region
 * @property-read Collection<int,Celestial> $celestials
 * @property-read Collection<int,Station> $stations
 * @property-read WormholeSystem|null $wormholeSystem
 * @property-read Sovereignty|null $sovereignty
 * @property-read Collection<int,MapSolarsystem> $mapSolarsystems
 * @property-read Collection<int,MapRouteSolarsystem> $mapRouteSolarsystems
 * @property-read Collection<int,Killmail> $killmails
 */
#[UseEloquentBuilder(SolarsystemBuilder::class)]
final class Solarsystem extends Model
{
    use HasSlug;

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
     * @return HasMany<Celestial,$this>
     */
    public function celestials(): HasMany
    {
        return $this->hasMany(Celestial::class, 'solarsystem_id');
    }

    /**
     * @return HasMany<Station,$this>
     */
    public function stations(): HasMany
    {
        return $this->hasMany(Station::class, 'solarsystem_id');
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

    /**
     * MapRouteSolarsystems related to this solarsystem.
     *
     * @return HasMany<MapRouteSolarsystem,$this>
     */
    public function mapRouteSolarsystems(): HasMany
    {
        return $this->hasMany(MapRouteSolarsystem::class, 'solarsystem_id');
    }

    /**
     * Killmails that are related to this solarsystem.
     *
     * @return HasMany<Killmail,$this>
     */
    public function killmails(): HasMany
    {
        return $this->hasMany(Killmail::class, 'solarsystem_id');
    }
}
