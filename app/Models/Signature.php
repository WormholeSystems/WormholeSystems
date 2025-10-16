<?php

declare(strict_types=1);

namespace App\Models;

use App\Enums\LifetimeStatus;
use App\Enums\MassStatus;
use App\Enums\ShipSize;
use Carbon\CarbonImmutable;
use DateTimeImmutable;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * Signature model representing a signature in the game.
 *
 * @property int $id
 * @property string $signature_id
 * @property int $map_solarsystem_id
 * @property int|null $map_connection_id
 * @property int|null $wormhole_id
 * @property int|null $signature_type_id
 * @property int|null $signature_category_id
 * @property string|null $raw_type_name
 * @property MassStatus|null $mass_status
 * @property LifetimeStatus $lifetime
 * @property DateTimeImmutable|string|null $lifetime_updated_at
 * @property ShipSize|null $ship_size
 * @property CarbonImmutable $created_at
 * @property CarbonImmutable $updated_at
 * @property-read MapSolarsystem $mapSolarsystem
 * @property-read MapConnection|null $mapConnection
 * @property-read Wormhole|null $wormhole
 * @property-read SignatureType|null $signatureType
 * @property-read SignatureCategory|null $signatureCategory
 */
final class Signature extends Model
{
    protected $casts = [
        'created_at' => 'immutable_datetime',
        'updated_at' => 'immutable_datetime',
        'mass_status' => MassStatus::class,
        'ship_size' => ShipSize::class,
        'lifetime' => LifetimeStatus::class,
        'lifetime_updated_at' => 'immutable_datetime',
    ];

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

    /**
     * Get the signature type associated with this signature.
     *
     * @return BelongsTo<SignatureType, $this>
     */
    public function signatureType(): BelongsTo
    {
        return $this->belongsTo(SignatureType::class);
    }

    /**
     * Get the signature category associated with this signature.
     *
     * @return BelongsTo<SignatureCategory, $this>
     */
    public function signatureCategory(): BelongsTo
    {
        return $this->belongsTo(SignatureCategory::class);
    }
}
