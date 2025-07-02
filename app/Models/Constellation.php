<?php

declare(strict_types=1);

namespace App\Models;

use App\Traits\HasSlug;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use NicolasKion\SDE\ClassResolver;

/**
 * @property int $id
 * @property string $name
 * @property int $region_id
 * @property string $type
 * @property-read Region $region
 * @property-read Collection<Solarsystem> $solarsystems
 * @property-read Collection<Station> $stations
 * @property-read Collection<Celestial> $celestials
 */
class Constellation extends Model
{
    use HasSlug;

    public $incrementing = false;

    protected $fillable = [
        'id',
        'name',
        'region_id',
        'type',
    ];

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
