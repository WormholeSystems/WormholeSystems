<?php

declare(strict_types=1);

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use NicolasKion\SDE\ClassResolver;

class Celestial extends Model
{
    public $incrementing = false;

    protected $fillable = [
        'name',
        'solarsystem_id',
        'constellation_id',
        'region_id',
        'parent_id',
        'type_id',
        'group_id',
    ];

    /**
     * @return BelongsTo<Region,$this>
     */
    public function region(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::region());
    }

    /**
     * @return BelongsTo<Constellation,$this>
     */
    public function constellation(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::constellation());
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
     * @return BelongsTo<Celestial,$this>
     */
    public function parent(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::celestial());
    }

    /**
     * @return HasMany<Station,$this>
     */
    public function stations(): HasMany
    {
        return $this->hasMany(ClassResolver::station(), 'parent_id');
    }
}
