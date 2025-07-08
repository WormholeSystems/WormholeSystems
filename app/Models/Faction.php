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
 * Faction model representing a faction in the game.
 *
 * @property int $id
 * @property string $name
 * @property string|null $description
 * @property int $corporation_id
 * @property int $militia_corporation_id
 * @property int $solarsystem_id
 * @property float $size_factor
 * @property int $station_count
 * @property int $station_system_count
 * @property bool $is_unique
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Corporation $militiaCorporation
 * @property-read Corporation $corporation
 * @property-read Collection<int,Corporation> $corporations
 * @property-read Collection<int,Character> $characters
 * @property-read Collection<int,Alliance> $alliances
 */
class Faction extends Model
{
    public $incrementing = false;

    /**
     * @return BelongsTo<Corporation,$this>
     */
    public function militiaCorporation(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::corporation());
    }

    /**
     * @return BelongsTo<Corporation,$this>
     */
    public function corporation(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::corporation());
    }

    /**
     * @return HasMany<Corporation,$this>
     */
    public function corporations(): HasMany
    {
        return $this->hasMany(ClassResolver::corporation());
    }

    /**
     * @return HasMany<Character,$this>
     */
    public function characters(): HasMany
    {
        return $this->hasMany(ClassResolver::character(), 'faction_id');
    }

    /**
     * @return HasMany<Alliance,$this>
     */
    public function alliances(): HasMany
    {
        return $this->hasMany(ClassResolver::alliance(), 'faction_id');
    }

    protected function casts(): array
    {
        return [
            'is_unique' => 'boolean',
        ];
    }
}
