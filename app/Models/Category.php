<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\HasMany;

/**
 * Category model representing a category in the game.
 *
 * @property int $id
 * @property string $name
 * @property int|null $icon_id
 * @property bool $published
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Collection<int,Group> $groups
 */
final class Category extends Model
{
    public $incrementing = false;

    /**
     * @return HasMany<Group,$this>
     */
    public function groups(): HasMany
    {
        return $this->hasMany(Group::class, 'category_id');
    }
}
