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
 * MetaGroup model representing a meta group in the game.
 *
 * @property int $id
 * @property string $name
 * @property string|null $description
 * @property int|null $icon_id
 * @property string|null $icon_suffix
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Collection<int,Type> $types
 * @property-read Icon|null $icon
 */
class MetaGroup extends Model
{
    public $incrementing = false;

    /**
     * @return HasMany<Type,$this>
     */
    public function types(): HasMany
    {
        return $this->hasMany(ClassResolver::type(), 'meta_group_id');
    }

    /**
     * @return BelongsTo<Icon,$this>
     */
    public function icon(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::icon());
    }
}
