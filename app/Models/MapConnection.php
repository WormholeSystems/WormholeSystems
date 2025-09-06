<?php

declare(strict_types=1);

namespace App\Models;

use App\Enums\MassStatus;
use App\Enums\ShipSize;
use Carbon\CarbonImmutable;
use Database\Factories\MapConnectionFactory;
use DateTimeImmutable;
use Illuminate\Database\Eloquent\Attributes\UseFactory;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;

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
 * @property DateTimeImmutable|string|null $marked_as_eol_at
 * @property CarbonImmutable $connected_at
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read MapSolarsystem $fromMapSolarsystem
 * @property-read MapSolarsystem $toMapSolarsystem
 * @property-read Map $map
 * @property-read Collection<int, Signature> $signatures
 */
#[UseFactory(MapConnectionFactory::class)]
final class MapConnection extends Model
{
    /** @use HasFactory<MapConnectionFactory> */
    use HasFactory;

    protected $casts = [
        'connected_at' => 'immutable_datetime',
        'created_at' => 'immutable_datetime',
        'updated_at' => 'immutable_datetime',
        'mass_status' => MassStatus::class,
        'ship_size' => ShipSize::class,
        'marked_as_eol_at' => 'immutable_datetime',
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

    /**
     * The signatures associated with this connection.
     *
     * @return HasMany<Signature, $this>
     */
    public function signatures(): HasMany
    {
        return $this->hasMany(Signature::class);
    }
}
