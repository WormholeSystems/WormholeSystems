<?php

declare(strict_types=1);

namespace App\Models;

use App\Traits\HasSlug;
use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\HasMany;

/**
 * Region model representing a region in the game.
 *
 * @property int $id
 * @property string $name
 * @property string $type
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Collection<int,Constellation> $constellations
 * @property-read Collection<int,Solarsystem> $solarsystems
 * @property-read Collection<int,Celestial> $celestials
 * @property-read Collection<int,Station> $stations
 */
final class Region extends Model
{
    use HasSlug;

    public $incrementing = false;

    /**
     * @return HasMany<Constellation,$this>
     */
    public function constellations(): HasMany
    {
        return $this->hasMany(Constellation::class, 'region_id');
    }

    /**
     * @return HasMany<Solarsystem,$this>
     */
    public function solarsystems(): HasMany
    {
        return $this->hasMany(Solarsystem::class, 'region_id');
    }

    /**
     * @return HasMany<Celestial,$this>
     */
    public function celestials(): HasMany
    {
        return $this->hasMany(Celestial::class, 'region_id');
    }

    /**
     * @return HasMany<Station,$this>
     */
    public function stations(): HasMany
    {
        return $this->hasMany(Station::class, 'region_id');
    }
}
