<?php

namespace App\Models;

use App\Enums\MassStatus;
use App\Enums\ShipSize;
use Carbon\CarbonImmutable;
use Database\Factories\MapConnectionFactory;
use Illuminate\Database\Eloquent\Attributes\UseFactory;
use Illuminate\Database\Eloquent\Factories\HasFactory;
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
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read MapSolarsystem $fromMapSolarsystem
 * @property-read MapSolarsystem $toMapSolarsystem
 * @property-read Map $map
 */
#[UseFactory(MapConnectionFactory::class)]
class MapConnection extends Model
{
    /** @use HasFactory<MapConnectionFactory> */
    use HasFactory;

    protected $casts = [
        'connected_at' => 'immutable_datetime',
        'created_at' => 'immutable_datetime',
        'updated_at' => 'immutable_datetime',
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

    /**
     * The map this connection belongs to.
     *
     * @return BelongsTo<Map, $this>
     */
    public function map(): BelongsTo
    {
        return $this->belongsTo(Map::class);
    }
}
