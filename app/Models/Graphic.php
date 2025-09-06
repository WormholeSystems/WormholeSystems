<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\HasMany;

/**
 * Graphic model representing a graphic in the game.
 *
 * @property int $id
 * @property string|null $sof_faction_name
 * @property string $file
 * @property string|null $sof_hull_name
 * @property string|null $sof_race_name
 * @property string|null $description
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Collection<int,Type> $types
 */
final class Graphic extends Model
{
    public $incrementing = false;

    /**
     * @return HasMany<Type,$this>
     */
    public function types(): HasMany
    {
        return $this->hasMany(Type::class, 'graphic_id');
    }
}
