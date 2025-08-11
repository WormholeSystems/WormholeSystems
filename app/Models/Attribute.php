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
 * Attribute model representing an attribute in the game.
 *
 * @property int $id
 * @property string $name
 * @property string $display_name
 * @property string|null $description
 * @property mixed $default_value
 * @property int|null $unit_id
 * @property int|null $icon_id
 * @property bool $high_is_good
 * @property bool $published
 * @property bool $stackable
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Collection<int,TypeAttribute> $typeAttributes
 * @property-read Icon|null $icon
 * @property-read Unit|null $unit
 */
final class Attribute extends Model
{
    public $incrementing = false;

    /**
     * @return HasMany<TypeAttribute,$this>
     */
    public function typeAttributes(): HasMany
    {
        return $this->hasMany(ClassResolver::typeAttribute(), 'attribute_id');
    }

    /**
     * @return BelongsTo<Unit,$this>
     */
    public function unit(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::unit());
    }

    /**
     * @return BelongsTo<Icon,$this>
     */
    public function icon(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::icon());
    }
}
