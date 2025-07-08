<?php

namespace App\Models;

use App\Enums\MapSolarsystemStatus;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Database\Eloquent\Relations\HasOne;

/**
 * Represents a solarsystem on a map
 *
 * @property int $id
 * @property int $map_id
 * @property int $solarsystem_id
 * @property string $alias
 * @property string $occupier_alias
 * @property float|null $position_x
 * @property float|null $position_y
 * @property MapSolarsystemStatus $status
 * @property bool $pinned
 * @property-read Solarsystem $solarsystem
 * @property-read Map $map
 * @property-read Collection<MapConnection> $connectionsTo
 * @property-read Collection<MapConnection> $connectionsFrom
 * @property-read WormholeSystem $wormholeSystem
 * @property-read Collection<Signature> $signatures
 */
class MapSolarsystem extends Model
{
    /**
     * @return BelongsTo<Solarsystem,$this>
     */
    public function solarsystem(): BelongsTo
    {
        return $this->belongsTo(Solarsystem::class);
    }

    /**
     * @return BelongsTo<Map,$this>
     */
    public function map(): BelongsTo
    {
        return $this->belongsTo(Map::class);
    }

    /**
     * @return HasMany<MapConnection,$this>
     */
    public function connectionsTo(): HasMany
    {
        return $this->hasMany(MapConnection::class, 'from_map_solarsystem_id');
    }

    /**
     * @return HasMany<MapConnection,$this>
     */
    public function connectionsFrom(): HasMany
    {
        return $this->hasMany(MapConnection::class, 'to_map_solarsystem_id');
    }

    /**
     * @return HasOne<WormholeSystem,$this>
     */
    public function wormholeSystem(): HasOne
    {
        return $this->hasOne(WormholeSystem::class, 'id', 'solarsystem_id');
    }

    /**
     * @return HasMany<Signature,$this>
     */
    public function signatures(): HasMany
    {
        return $this->hasMany(Signature::class, 'map_solarsystem_id');
    }

    public ?Collection $connections {
        get {
            $mergedConnections = $this->connectionsTo->merge($this->connectionsFrom);

            $this->setRelation('connections', $mergedConnections);

            return $mergedConnections;
        }
    }

    protected function casts(): array
    {
        return [
            'status' => MapSolarsystemStatus::class,
        ];
    }
}
