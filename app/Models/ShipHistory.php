<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * Class ShipHistory
 *
 * @property int $id
 * @property int $ship_id
 * @property int $ship_type_id
 * @property int $character_id
 * @property string $name
 * @property CarbonImmutable|string $created_at
 * @property CarbonImmutable|string $updated_at
 * @property-read Type $shipType
 * @property-read Character $character
 */
final class ShipHistory extends Model
{
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
        ];
    }
}
