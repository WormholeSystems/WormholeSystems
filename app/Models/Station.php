<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * Station model representing a station in the game.
 *
 * @property int $id
 * @property string $name
 * @property int $solarsystem_id
 * @property int $constellation_id
 * @property int $region_id
 * @property int|null $parent_id
 * @property int $type_id
 * @property int $group_id
 * @property int|null $operation_id
 * @property int|null $owner_id
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Solarsystem $solarsystem
 * @property-read Constellation $constellation
 * @property-read Celestial|null $parent
 * @property-read Type $type
 * @property-read Group $group
 * @property-read Region $region
 * @property-read StationOperation|null $operation
 * @property-read Corporation|null $owner
 */
final class Station extends Model
{
    public $incrementing = false;

    /**
     * @return BelongsTo<Solarsystem,$this>
     */
    public function solarsystem(): BelongsTo
    {
        return $this->belongsTo(Solarsystem::class);
    }

    /**
     * @return BelongsTo<Constellation,$this>
     */
    public function constellation(): BelongsTo
    {
        return $this->belongsTo(Constellation::class);
    }

    /**
     * @return BelongsTo<Celestial,$this>
     */
    public function parent(): BelongsTo
    {
        return $this->belongsTo(Celestial::class);
    }

    /**
     * @return BelongsTo<Type,$this>
     */
    public function type(): BelongsTo
    {
        return $this->belongsTo(Type::class);
    }

    /**
     * @return BelongsTo<Group,$this>
     */
    public function group(): BelongsTo
    {
        return $this->belongsTo(Group::class);
    }

    /**
     * @return BelongsTo<Region,$this>
     */
    public function region(): BelongsTo
    {
        return $this->belongsTo(Region::class);
    }

    /**
     * @return BelongsTo<StationOperation,$this>
     */
    public function operation(): BelongsTo
    {
        return $this->belongsTo(StationOperation::class, 'operation_id');
    }

    /**
     * @return BelongsTo<Corporation,$this>
     */
    public function owner(): BelongsTo
    {
        return $this->belongsTo(Corporation::class, 'owner_id');
    }
}
