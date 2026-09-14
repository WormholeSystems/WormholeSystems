<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * A finalized, frozen maintainer leaderboard for one map and one UTC calendar month.
 * Point weights and thresholds are mutable map settings, so once a period is over its
 * report is computed once and never recomputed -- see MaintainerLeaderboard::aggregate().
 *
 * @property int $id
 * @property int $map_id
 * @property string $period
 * @property array{characters: list<array<string, mixed>>, settings: array<string, int>} $payload
 * @property CarbonImmutable $generated_at
 * @property CarbonImmutable|null $discord_posted_at
 * @property CarbonImmutable $created_at
 * @property CarbonImmutable $updated_at
 * @property-read Map $map
 */
final class MapMaintainerReport extends Model
{
    protected $casts = [
        'payload' => 'array',
        'generated_at' => 'immutable_datetime',
        'discord_posted_at' => 'immutable_datetime',
    ];

    /**
     * @return BelongsTo<Map, $this>
     */
    public function map(): BelongsTo
    {
        return $this->belongsTo(Map::class);
    }
}
