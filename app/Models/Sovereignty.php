<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * The sovereignty of a solar system in the game.
 *
 * @property int $id
 * @property int $solarsystem_id
 * @property int|null $alliance_id
 * @property int|null $corporation_id
 * @property int|null $faction_id
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Alliance|null $alliance
 * @property-read Corporation|null $corporation
 * @property-read Faction|null $faction
 * @property-read Solarsystem $solarsystem
 */
final class Sovereignty extends Model
{
    /**
     * The solarsystem that this sovereignty belongs to.
     *
     * @return BelongsTo<Solarsystem,$this>
     */
    public function solarsystem(): BelongsTo
    {
        return $this->belongsTo(Solarsystem::class);
    }

    /**
     * The alliance that owns this sovereignty.
     *
     * @return BelongsTo<Alliance,$this>
     */
    public function alliance(): BelongsTo
    {
        return $this->belongsTo(Alliance::class);
    }

    /**
     * The corporation that owns this sovereignty.
     *
     * @return BelongsTo<Corporation,$this>
     */
    public function corporation(): BelongsTo
    {
        return $this->belongsTo(Corporation::class);
    }

    /**
     * The faction that owns this sovereignty.
     *
     * @return BelongsTo<Faction,$this>
     */
    public function faction(): BelongsTo
    {
        return $this->belongsTo(Faction::class);
    }
}
