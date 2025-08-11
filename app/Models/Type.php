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
 * Type model representing a type in the game.
 *
 * @property int $id
 * @property string $name
 * @property string|null $description
 * @property int|null $graphic_id
 * @property int $group_id
 * @property int|null $icon_id
 * @property int|null $market_group_id
 * @property int|null $meta_group_id
 * @property int|null $race_id
 * @property bool $published
 * @property float|null $capacity
 * @property float|null $mass
 * @property float|null $base_price
 * @property float|null $volume
 * @property float|null $packaged_volume
 * @property float|null $radius
 * @property int|null $portion_size
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Collection<int,TypeAttribute> $typeAttributes
 * @property-read Race|null $race
 * @property-read Group $group
 * @property-read Icon|null $icon
 * @property-read MarketGroup|null $marketGroup
 * @property-read MetaGroup|null $metaGroup
 * @property-read Graphic|null $graphic
 */
final class Type extends Model
{
    public $incrementing = false;

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

    protected function casts(): array
    {
        return [
            'published' => 'boolean',
        ];
    }
}
