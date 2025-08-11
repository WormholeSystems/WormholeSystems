<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * Wormhole model representing a wormhole in the game.
 *
 * @property int $id
 * @property string $name
 * @property float $total_mass
 * @property float $maximum_jump_mass
 * @property string $ship_size
 * @property int $maximum_lifetime
 * @property string $leads_to
 * @property int $type_id
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Type $type
 */
final class Wormhole extends Model
{
    /**
     * The type of the wormhole.
     *
     * @return BelongsTo<Type, $this>
     */
    public function type(): BelongsTo
    {
        return $this->belongsTo(Type::class);
    }
}
