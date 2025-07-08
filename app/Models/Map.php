<?php

namespace App\Models;

use App\Traits\HasSlug;
use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\HasMany;

/**
 * A player map with solar systems and their connections.
 *
 * @property int $id
 * @property string $name
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Collection<int,MapSolarsystem> $mapSolarsystems
 * @property-read Collection<int,MapConnection> $mapConnections
 */
class Map extends Model
{
    use HasSlug;

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
}
