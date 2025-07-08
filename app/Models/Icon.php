<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\HasMany;
use NicolasKion\SDE\ClassResolver;

/**
 * Icon model representing an icon in the game.
 *
 * @property int $id
 * @property string $file
 * @property string|null $description
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Collection<int,Type> $types
 * @property-read Collection<int,Attribute> $attributes
 * @property-read Collection<int,MarketGroup> $marketGroups
 * @property-read Collection<int,MetaGroup> $metaGroups
 * @property-read Collection<int,Group> $groups
 * @property-read Collection<int,Category> $categories
 */
class Icon extends Model
{
    public $incrementing = false;

    /**
     * @return HasMany<Type,$this>
     */
    public function types(): HasMany
    {
        return $this->hasMany(ClassResolver::type(), 'icon_id');
    }

    /**
     * @return HasMany<Attribute,$this>
     */
    public function attributes(): HasMany
    {
        return $this->hasMany(ClassResolver::attribute(), 'icon_id');
    }

    /**
     * @return HasMany<MarketGroup,$this>
     */
    public function marketGroups(): HasMany
    {
        return $this->hasMany(ClassResolver::marketGroup(), 'icon_id');
    }

    /**
     * @return HasMany<MetaGroup,$this>
     */
    public function metaGroups(): HasMany
    {
        return $this->hasMany(ClassResolver::metaGroup(), 'icon_id');
    }

    /**
     * @return HasMany<Group,$this>
     */
    public function groups(): HasMany
    {
        return $this->hasMany(ClassResolver::group(), 'icon_id');
    }

    /**
     * @return HasMany<Category,$this>
     */
    public function categories(): HasMany
    {
        return $this->hasMany(ClassResolver::category(), 'icon_id');
    }
}
