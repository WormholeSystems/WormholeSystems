<?php

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Support\Fluent;
use Staudenmeir\EloquentJsonRelations\HasJsonRelationships;

/**
 * Killmail model representing a killmail in the game.
 *
 * @property int $id
 * @property string $hash
 * @property int $solarsystem_id
 * @property string|CarbonImmutable $time
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Fluent $data
 * @property-read Fluent $zkb
 * @property-read Type $shipType
 * @property-read Solarsystem $solarsystem
 */
class Killmail extends Model
{
    use HasJsonRelationships;

    public $incrementing = false;

    protected $casts = [
        'id' => 'integer',
        'solarsystem_id' => 'integer',
        'data' => 'json',
        'zkb' => 'json',
        'time' => 'datetime',
    ];

    /**
     * The solar system this killmail occurred in.
     *
     * @return BelongsTo<Solarsystem, $this>
     */
    public function solarsystem(): BelongsTo
    {
        return $this->belongsTo(Solarsystem::class);
    }

    /**
     * @return BelongsTo<Type,$this>
     */
    public function shipType(): BelongsTo
    {
        return $this->belongsTo(Type::class, 'data->victim->ship_type_id', 'id');
    }
}
