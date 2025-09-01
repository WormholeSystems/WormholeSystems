<?php

declare(strict_types=1);

namespace App\Models;

use App\Enums\MassStatus;
use App\Enums\ShipSize;
use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * Signature model representing a signature in the game.
 *
 * @property int $id
 * @property string $signature_id
 * @property string $type
 * @property string|null $category
 * @property int $map_solarsystem_id
 * @property int|null $map_connection_id
 * @property int|null $wormhole_id
 * @property MassStatus|null $mass_status
 * @property bool|null $is_eol
 * @property ShipSize|null $ship_size
 * @property CarbonImmutable $created_at
 * @property CarbonImmutable $updated_at
 * @property-read MapSolarsystem $mapSolarsystem
 * @property-read MapConnection|null $mapConnection
 * @property-read Wormhole|null $wormhole
 */
final class Signature extends Model
{
    protected $casts = [
        'created_at' => 'immutable_datetime',
        'updated_at' => 'immutable_datetime',
        'mass_status' => MassStatus::class,
        'ship_size' => ShipSize::class,
        'is_eol' => 'boolean',
    ];

    public static function typeToWormhole(?string $type = null): ?Wormhole
    {
        if ($type === null || $type === '') {
            return null;
        }

        $name = str($type)->explode(' - ')->first();

        return Wormhole::query()->where('name', $name)->first();
    }

    /**
     * @return BelongsTo<MapSolarsystem,$this>
     */
    public function mapSolarsystem(): BelongsTo
    {
        return $this->belongsTo(MapSolarsystem::class);
    }

    /**
     * @return BelongsTo<MapConnection,$this>
     */
    public function mapConnection(): BelongsTo
    {
        return $this->belongsTo(MapConnection::class, 'map_connection_id');
    }

    /**
     * Get the wormhole associated with this signature.
     *
     * @return BelongsTo<Wormhole, $this>
     */
    public function wormhole(): BelongsTo
    {
        return $this->belongsTo(Wormhole::class);
    }
}
