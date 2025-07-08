<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use NicolasKion\SDE\ClassResolver;

/**
 * Group model representing a group in the game.
 *
 * @property int $id
 * @property string $name
 * @property int $category_id
 * @property int|null $icon_id
 * @property bool $published
 * @property bool $use_base_price
 * @property bool $anchored
 * @property bool $anchorable
 * @property bool $fittable_non_singleton
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Icon|null $icon
 * @property-read Category $category
 * @property-read Collection<int,Type> $types
 * @property-read Collection<int,Celestial> $celestials
 * @property-read Collection<int,Station> $stations
 */
class Group extends Model
{
    public $incrementing = false;

    /**
     * @return BelongsTo<Icon,$this>
     */
    public function icon(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::icon());
    }

    /**
     * @return BelongsTo<Category,$this>
     */
    public function category(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::category());
    }

    /**
     * @return HasMany<Type,$this>
     */
    public function types(): HasMany
    {
        return $this->hasMany(ClassResolver::type(), 'group_id');
    }

    /**
     * @return HasMany<Celestial,$this>
     */
    public function celestials(): HasMany
    {
        return $this->hasMany(ClassResolver::celestial(), 'group_id');
    }

    /**
     * @return HasMany<Station,$this>
     */
    public function stations(): HasMany
    {
        return $this->hasMany(ClassResolver::station(), 'group_id');
    }

    protected function casts(): array
    {
        return [
            'published' => 'boolean',
            'use_base_price' => 'boolean',
            'anchored' => 'boolean',
            'anchorable' => 'boolean',
            'fittable_non_singleton' => 'boolean',
        ];
    }
}
