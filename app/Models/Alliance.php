<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Database\Eloquent\Relations\MorphMany;
use NicolasKion\SDE\ClassResolver;

/**
 * Represents an alliance in the game.
 *
 * @property int $id
 * @property string $name
 * @property string $ticker
 * @property int $creator_id
 * @property int $creator_corporation_id
 * @property int|null $faction_id
 * @property string|CarbonImmutable $date_founded
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Character $creator
 * @property-read Corporation $creatorCorporation
 * @property-read Faction|null $faction
 * @property-read Collection<int,Corporation> $corporations
 * @property-read Collection<int,Character> $characters
 * @property-read Collection<int,MapAccess> $mapAccesses
 */
class Alliance extends Model
{
    use HasFactory;

    public $incrementing = false;

    /**
     * @return BelongsTo<Character,$this>
     */
    public function creator(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::character());
    }

    /**
     * @return BelongsTo<Corporation,$this>
     */
    public function creatorCorporation(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::corporation());
    }

    /**
     * @return BelongsTo<Faction,$this>
     */
    public function faction(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::faction());
    }

    /**
     * @return HasMany<Corporation,$this>
     */
    public function corporations(): HasMany
    {
        return $this->hasMany(ClassResolver::corporation(), 'alliance_id');
    }

    /**
     * @return HasMany<Character,$this>
     */
    public function characters(): HasMany
    {
        return $this->hasMany(ClassResolver::character(), 'alliance_id');
    }

    /**
     * @return MorphMany<MapAccess,$this>
     */
    public function mapAccesses(): MorphMany
    {
        return $this->morphMany(MapAccess::class, 'accessible');
    }

    protected function casts(): array
    {
        return [
            'date_founded' => 'datetime',
        ];
    }
}
