<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Database\Factories\MapConnectionJumpFactory;
use Illuminate\Database\Eloquent\Attributes\UseFactory;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * A ship jump through a map connection, observed via character location tracking.
 *
 * Rows with a null map_connection_id are pending: the jump was seen before the
 * connection existed on the map and is claimed once the connection is created.
 *
 * @property int $id
 * @property int $map_id
 * @property int|null $map_connection_id
 * @property int|null $character_id
 * @property int $from_solarsystem_id
 * @property int $to_solarsystem_id
 * @property int|null $ship_type_id
 * @property string|null $ship_name
 * @property int $mass
 * @property bool $is_manual
 * @property CarbonImmutable|string $created_at
 * @property CarbonImmutable|string $updated_at
 * @property-read Map $map
 * @property-read MapConnection|null $mapConnection
 * @property-read Character|null $character
 * @property-read Type|null $shipType
 */
#[UseFactory(MapConnectionJumpFactory::class)]
final class MapConnectionJump extends Model
{
    /** @use HasFactory<MapConnectionJumpFactory> */
    use HasFactory;

    /**
     * @return BelongsTo<Map, $this>
     */
    public function map(): BelongsTo
    {
        return $this->belongsTo(Map::class);
    }

    /**
     * @return BelongsTo<MapConnection, $this>
     */
    public function mapConnection(): BelongsTo
    {
        return $this->belongsTo(MapConnection::class);
    }

    /**
     * @return BelongsTo<Character, $this>
     */
    public function character(): BelongsTo
    {
        return $this->belongsTo(Character::class);
    }

    /**
     * @return BelongsTo<Type, $this>
     */
    public function shipType(): BelongsTo
    {
        return $this->belongsTo(Type::class, 'ship_type_id');
    }

    protected function casts(): array
    {
        return [
            'created_at' => 'immutable_datetime',
            'updated_at' => 'immutable_datetime',
            'mass' => 'integer',
            'is_manual' => 'boolean',
        ];
    }
}
