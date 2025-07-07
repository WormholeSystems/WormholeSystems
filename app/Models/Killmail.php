<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Support\Fluent;
use Staudenmeir\EloquentJsonRelations\HasJsonRelationships;

/**
 * @property int $id
 * @property string $hash
 * @property int $solarsystem_id
 * @property string $time
 * @property-read Fluent $data
 * @property-read Fluent $zkb
 * @property-read Type $shipType
 * @property-read Solarsystem $solarsystem
 */
class Killmail extends Model
{
    use HasJsonRelationships;

    public $incrementing = false;

    protected $fillable = [
        'id',
        'hash',
        'solarsystem_id',
        'data',
        'time',
        'zkb',
    ];

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

    public function shipType(): BelongsTo
    {
        return $this->belongsTo(Type::class, 'data->victim->ship_type_id', 'id');
    }
}
