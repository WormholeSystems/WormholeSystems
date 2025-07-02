<?php

declare(strict_types=1);

namespace App\Models;

use App\Traits\HasSlug;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Database\Eloquent\Relations\HasOne;
use NicolasKion\SDE\ClassResolver;

/**
 * @property int $id
 * @property string $name
 * @property int $constellation_id
 * @property int $region_id
 * @property float $security
 * @property float $pos_x
 * @property float $pos_y
 * @property float $pos_z
 * @property string $type
 * @property-read Constellation $constellation
 * @property-read Region $region
 * @property-read Collection<Celestial> $celestials
 * @property-read Collection<Station> $stations
 * @property-read WormholeSystem|null $wormholeSystem
 */
class Solarsystem extends Model
{
    use HasSlug;

    public $incrementing = false;

    protected $fillable = [
        'id',
        'name',
        'constellation_id',
        'region_id',
        'security',
        'pos_x',
        'pos_y',
        'pos_z',
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
     */
    public function mapSolarsystems(): HasMany
    {
        return $this->hasMany(MapSolarsystem::class, 'solarsystem_id');
    }
}
