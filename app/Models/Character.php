<?php

declare(strict_types=1);

namespace App\Models;

use App\Builders\CharacterBuilder;
use Carbon\CarbonImmutable;
use Database\Factories\CharacterFactory;
use Illuminate\Database\Eloquent\Attributes\UseEloquentBuilder;
use Illuminate\Database\Eloquent\Attributes\UseFactory;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Database\Eloquent\Relations\HasOne;
use Illuminate\Database\Eloquent\Relations\MorphMany;
use NicolasKion\Esi\Enums\EsiScope;
use NicolasKion\Esi\Interfaces\EsiToken;
use Staudenmeir\EloquentHasManyDeep\HasManyDeep;
use Staudenmeir\EloquentHasManyDeep\HasRelationships;

/**
 * Character model representing a character in the game.
 *
 * @property int $id
 * @property string $name
 * @property string|null $description
 * @property int $race_id
 * @property int $bloodline_id
 * @property int $corporation_id
 * @property int|null $faction_id
 * @property int|null $alliance_id
 * @property float $security_status
 * @property string $gender
 * @property string|CarbonImmutable $birthday
 * @property string|null $title
 * @property int|null $user_id
 * @property string|null $character_owner_hash
 * @property array|null $route
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Race $race
 * @property-read Bloodline $bloodline
 * @property-read Corporation|null $corporation
 * @property-read Faction|null $faction
 * @property-read Alliance|null $alliance
 * @property-read User|null $user
 * @property-read Collection<int,\App\Models\EsiToken> $esiTokens
 * @property-read Type|null $shipType
 * @property-read Solarsystem|null $solarsystem
 * @property-read CharacterStatus|null $characterStatus
 * @property-read Collection<int,MapAccess> $mapAccesses
 * @property-read Collection<int,\App\Models\EsiScope> $esiScopes
 * @property-read Collection<int,Map> $mapsOwned
 */
#[UseFactory(CharacterFactory::class)]
#[UseEloquentBuilder(CharacterBuilder::class)]
final class Character extends Model implements \NicolasKion\Esi\Interfaces\Character
{
    /** @use HasFactory<CharacterFactory> */
    use HasFactory;

    use HasRelationships;

    public $incrementing = false;

    /**
     * @return BelongsTo<Race,$this>
     */
    public function race(): BelongsTo
    {
        return $this->belongsTo(Race::class);
    }

    /**
     * @return BelongsTo<Bloodline,$this>
     */
    public function bloodline(): BelongsTo
    {
        return $this->belongsTo(Bloodline::class);
    }

    /**
     * @return BelongsTo<Corporation,$this>
     */
    public function corporation(): BelongsTo
    {
        return $this->belongsTo(Corporation::class);
    }

    /**
     * @return BelongsTo<Faction,$this>
     */
    public function faction(): BelongsTo
    {
        return $this->belongsTo(Faction::class);
    }

    /**
     * @return BelongsTo<Alliance,$this>
     */
    public function alliance(): BelongsTo
    {
        return $this->belongsTo(Alliance::class);
    }

    /**
     * @return BelongsTo<User,$this>
     */
    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class);
    }

    /**
     * Get the ESI tokens associated with the character.
     *
     * @return HasMany<\App\Models\EsiToken,$this>
     */
    public function esiTokens(): HasMany
    {
        return $this->hasMany(\App\Models\EsiToken::class, 'character_id');
    }

    /**
     * @return HasOne<CharacterStatus,$this>
     */
    public function characterStatus(): HasOne
    {
        return $this->hasOne(CharacterStatus::class, 'character_id');
    }

    /**
     * @return MorphMany<MapAccess,$this>
     */
    public function mapAccesses(): MorphMany
    {
        return $this->morphMany(MapAccess::class, 'accessible');
    }

    public function mapsOwned(): HasManyDeep
    {
        return $this->hasManyDeepFromRelations(
            $this->mapAccesses(),
            new MapAccess()->map()
        );
    }

    public function getEsiTokenWithScope(EsiScope $scope): ?EsiToken
    {
        return $this->esiTokens()
            ->whereRelation('esiScopes', 'name', $scope)
            ->first();
    }

    public function esiScopes(): HasManyDeep
    {
        return $this->hasManyDeepFromRelations(
            $this->esiTokens(),
            new \App\Models\EsiToken()->esiScopes()
        );
    }

    public function getId(): int
    {
        return $this->id;
    }

    public function getCorporationId(): int
    {
        return $this->corporation_id;
    }

    protected function casts(): array
    {
        return [
            'birthday' => 'immutable_datetime',
        ];
    }
}
