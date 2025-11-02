<?php

declare(strict_types=1);

namespace App\Models;

use App\Builders\CharacterStatusBuilder;
use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Attributes\UseEloquentBuilder;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * CharacterStatus model representing the status of a character.
 *
 * @property int $id
 * @property int $character_id
 * @property int $solarsystem_id
 * @property int $station_id
 * @property int $structure_id
 * @property string $ship_name
 * @property int $ship_type_id
 * @property int $ship_item_id
 * @property bool $is_online
 * @property CarbonImmutable|null $event_queued_at
 * @property CarbonImmutable|null $last_online_at
 * @property CarbonImmutable|null $online_last_checked_at
 * @property CarbonImmutable|null $location_last_checked_at
 * @property-read CarbonImmutable $created_at
 * @property-read CarbonImmutable $updated_at
 * @property-read Character $character
 * @property-read Solarsystem|null $solarsystem
 * @property-read Station|null $station
 * @property-read Type|null $shipType
 */
#[UseEloquentBuilder(CharacterStatusBuilder::class)]
final class CharacterStatus extends Model
{
    /**
     * @return BelongsTo<Character, $this>
     */
    public function character(): BelongsTo
    {
        return $this->belongsTo(Character::class);
    }

    /**
     * @return BelongsTo<Solarsystem, $this>
     */
    public function solarsystem(): BelongsTo
    {
        return $this->belongsTo(Solarsystem::class);
    }

    /**
     * @return BelongsTo<Station, $this>
     */
    public function station(): BelongsTo
    {
        return $this->belongsTo(Station::class);
    }

    /**
     * @return BelongsTo<Type, $this>
     */
    public function shipType(): BelongsTo
    {
        return $this->belongsTo(Type::class, 'ship_type_id');
    }

    public function newEloquentBuilder($query): CharacterStatusBuilder
    {

        return new CharacterStatusBuilder($query);
    }

    protected function casts(): array
    {
        return [
            'event_queued_at' => 'immutable_datetime',
            'last_online_at' => 'immutable_datetime',
            'online_last_checked_at' => 'immutable_datetime',
            'location_last_checked_at' => 'immutable_datetime',
        ];
    }
}
