<?php

namespace App\Models;

use App\Enums\MassStatus;
use App\Enums\ShipSize;
use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * Represents a connection between solar systems in a map.
 *
 * @property int $id
 * @property int $map_id
 * @property int $from_map_solarsystem_id
 * @property int $to_map_solarsystem_id
 * @property int|null $wormhole_id
 * @property string|MassStatus $mass_status
 * @property string|ShipSize $ship_size
 * @property bool $is_eol
 * @property CarbonImmutable $connected_at
 * @property-read MapSolarsystem $fromMapSolarsystem
 * @property-read MapSolarsystem $toMapSolarsystem
 */
class MapConnection extends Model
{
    protected $fillable = [
        'map_id',
        'from_map_solarsystem_id',
        'to_map_solarsystem_id',
        'wormhole_id',
        'mass_status',
        'ship_size',
        'is_eol',
        'connected_at',
    ];

    protected $casts = [
        'connected_at' => 'immutable_datetime',
        'mass_status' => MassStatus::class,
        'ship_size' => ShipSize::class,
        'is_eol' => 'boolean',
    ];

    /**
     * The solar system this connection is from.
     *
     * @return BelongsTo<MapSolarsystem, $this>
     */
    public function fromMapSolarsystem(): BelongsTo
    {
        return $this->belongsTo(MapSolarsystem::class);
    }

    /**
     * The solar system this connection is to.
     *
     * @return BelongsTo<MapSolarsystem, $this>
     */
    public function toMapSolarsystem(): BelongsTo
    {
        return $this->belongsTo(MapSolarsystem::class);
    }
}
