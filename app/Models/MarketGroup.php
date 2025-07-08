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
 * MarketGroup model representing a market group in the game.
 *
 * @property int $id
 * @property string $name
 * @property string|null $description
 * @property bool $has_types
 * @property int|null $icon_id
 * @property int|null $parent_id
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Collection<int,Type> $types
 * @property-read Icon|null $icon
 * @property-read MarketGroup|null $parent
 * @property-read Collection<int,MarketGroup> $children
 */
class MarketGroup extends Model
{
    public $incrementing = false;

    /**
     * @return HasMany<Type,$this>
     */
    public function types(): HasMany
    {
        return $this->hasMany(ClassResolver::type(), 'market_group_id');
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
    public function parent(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::marketGroup());
    }

    /**
     * @return HasMany<MarketGroup,$this>
     */
    public function children(): HasMany
    {
        return $this->hasMany(ClassResolver::marketGroup(), 'parent_id');
    }

    protected function casts(): array
    {
        return [
            'has_types' => 'bool',
        ];
    }
}
