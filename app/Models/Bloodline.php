<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;

/**
 * Bloodline model representing a character's bloodline in the game.
 *
 * @property int $id
 * @property string $name
 * @property string|null $description
 * @property int $race_id
 * @property int $ship_type_id
 * @property int $willpower
 * @property int $perception
 * @property int $charisma
 * @property int $intelligence
 * @property int $memory
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Type $shipType
 * @property-read Collection<int,Character> $characters
 */
final class Bloodline extends Model
{
    public $incrementing = false;

    /**
     * @return BelongsTo<Race,$this>
     */
    public function race(): BelongsTo
    {
        return $this->belongsTo(Race::class);
    }

    /**
     * @return BelongsTo<Type,$this>
     */
    public function shipType(): BelongsTo
    {
        return $this->belongsTo(Type::class);
    }

    /**
     * @return HasMany<Character,$this>
     */
    public function characters(): HasMany
    {
        return $this->hasMany(Character::class, 'bloodline_id');
    }
}
