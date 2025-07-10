<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Database\Eloquent\Relations\MorphMany;
use NicolasKion\SDE\ClassResolver;

/**
 * Corporation model representing a corporation in the game.
 *
 * @property int $id
 * @property string $name
 * @property int $ceo_id
 * @property int $creator_id
 * @property int|null $faction_id
 * @property int|null $home_station_id
 * @property int $member_count
 * @property int $shares
 * @property string|CarbonImmutable $date_founded
 * @property string|null $description
 * @property string|null $url
 * @property string $ticker
 * @property float $tax_rate
 * @property bool $war_eligible
 * @property bool $npc
 * @property int|null $alliance_id
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Character $ceo
 * @property-read Character $creator
 * @property-read Faction|null $faction
 * @property-read Station|null $homeStation
 * @property-read Alliance|null $alliance
 * @property-read Collection<int,Character> $characters
 * @property-read Collection<int,MapAccess> $mapAccesses
 */
class Corporation extends Model
{
    public $incrementing = false;

    /**
     * @return BelongsTo<Character,$this>
     */
    public function ceo(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::character());
    }

    /**
     * @return BelongsTo<Character,$this>
     */
    public function creator(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::character());
    }

    /**
     * @return BelongsTo<Faction,$this>
     */
    public function faction(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::faction());
    }

    /**
     * @return BelongsTo<Station,$this>
     */
    public function homeStation(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::station());
    }

    /**
     * @return BelongsTo<Alliance, $this>
     */
    public function alliance(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::alliance());
    }

    /**
     * @return HasMany<Character,$this>
     */
    public function characters(): HasMany
    {
        return $this->hasMany(ClassResolver::character(), 'corporation_id');
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
            'date_founded' => 'immutable_datetime',
            'war_eligible' => 'boolean',
            'npc' => 'boolean',
        ];
    }
}
