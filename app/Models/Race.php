<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\HasMany;

/**
 * Race model representing a race in the game.
 *
 * @property int $id
 * @property string $name
 * @property string|null $description
 * @property string|null $short_description
 * @property int|null $icon_id
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Collection<int,Type> $types
 * @property-read Collection<int,Character> $characters
 * @property-read Collection<int,Bloodline> $bloodlines
 */
final class Race extends Model
{
    public $incrementing = false;

    /**
     * @return HasMany<Type,$this>
     */
    public function types(): HasMany
    {
        return $this->hasMany(Type::class, 'race_id');
    }

    /**
     * @return HasMany<Character,$this>
     */
    public function characters(): HasMany
    {
        return $this->hasMany(Character::class, 'race_id');
    }

    /**
     * @return HasMany<Bloodline,$this>
     */
    public function bloodlines(): HasMany
    {
        return $this->hasMany(Bloodline::class, 'race_id');
    }
}
