<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use NicolasKion\SDE\ClassResolver;

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
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Solarsystem $solarsystem
 * @property-read Constellation $constellation
 * @property-read Celestial|null $parent
 * @property-read Type $type
 * @property-read Group $group
 * @property-read Region $region
 */
class Station extends Model
{
    public $incrementing = false;

    /**
     * @return BelongsTo<Solarsystem,$this>
     */
    public function solarsystem(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::solarsystem());
    }

    /**
     * @return BelongsTo<Constellation,$this>
     */
    public function constellation(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::constellation());
    }

    /**
     * @return BelongsTo<Celestial,$this>
     */
    public function parent(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::celestial());
    }

    /**
     * @return BelongsTo<Type,$this>
     */
    public function type(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::type());
    }

    /**
     * @return BelongsTo<Group,$this>
     */
    public function group(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::group());
    }

    /**
     * @return BelongsTo<Region,$this>
     */
    public function region(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::region());
    }
}
