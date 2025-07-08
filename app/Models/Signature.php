<?php

namespace App\Models;

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
 * @property string|null $name
 * @property CarbonImmutable $created_at
 * @property CarbonImmutable $updated_at
 * @property-read MapSolarsystem $mapSolarsystem
 */
class Signature extends Model
{
    protected $casts = [
        'created_at' => 'immutable_datetime',
        'updated_at' => 'immutable_datetime',
    ];

    /**
     * @return BelongsTo<MapSolarsystem,$this>
     */
    public function mapSolarsystem(): BelongsTo
    {
        return $this->belongsTo(MapSolarsystem::class);
    }
}
