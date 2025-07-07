<?php

namespace App\Models;

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
 * @property-read Type $type
 */
class Wormhole extends Model
{
    protected $fillable = [
        'name',
        'total_mass',
        'maximum_jump_mass',
        'ship_size',
        'maximum_lifetime',
        'leads_to',
        'type_id',
    ];

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
