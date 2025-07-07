<?php

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * @property int $id
 * @property string $signature_is
 * @property string $type
 * @property string|null $category
 * @property string|null $name
 * @property CarbonImmutable $created_at
 * @property CarbonImmutable $updated_at
 */
class Signature extends Model
{
    protected $fillable = [
        'signature_id',
        'type',
        'category',
        'name',
    ];

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
