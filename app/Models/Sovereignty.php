<?php

namespace App\Models;

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
 * @property-read Alliance|null $alliance
 * @property-read Corporation|null $corporation
 * @property-read Faction|null $faction
 * @property-read SolarSystem $solarsystem
 */
class Sovereignty extends Model
{
    protected $fillable = [
        'solarsystem_id',
        'alliance_id',
        'corporation_id',
        'faction_id',
    ];

    /**
     * The solarsystem that this sovereignty belongs to.
     *
     * @return BelongsTo<Alliance,$this>
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
