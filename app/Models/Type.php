<?php

declare(strict_types=1);

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use NicolasKion\SDE\ClassResolver;

class Type extends Model
{
    public $incrementing = false;

    protected $fillable = [
        'id',
        'name',
        'description',
        'graphic_id',
        'group_id',
        'icon_id',
        'market_group_id',
        'meta_group_id',
        'race_id',
        'published',
        'capacity',
        'mass',
        'base_price',
        'volume',
        'packaged_volume',
        'radius',
        'portion_size',
    ];

    /**
     * @return HasMany<TypeAttribute,$this>
     */
    public function typeAttributes(): HasMany
    {
        return $this->hasMany(ClassResolver::typeAttribute(), 'type_id');
    }

    /**
     * @return BelongsTo<Race,$this>
     */
    public function race(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::race());
    }

    /**
     * @return BelongsTo<Group,$this>
     */
    public function group(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::group());
    }

    /**
     * @return BelongsTo<Icon,$this>
     */
    public function icon(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::icon());
    }

    /**
     * @return BelongsTo<MarketGroup,$this>
     */
    public function marketGroup(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::marketGroup());
    }

    /**
     * @return BelongsTo<MetaGroup,$this>
     */
    public function metaGroup(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::metaGroup());
    }

    /**
     * @return BelongsTo<Graphic,$this>
     */
    public function graphic(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::graphic());
    }
}
