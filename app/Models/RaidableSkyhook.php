<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * A skyhook structure currently vulnerable (or shortly to be vulnerable) to theft.
 *
 * @property int $planet_id
 * @property int $solarsystem_id
 * @property string|CarbonImmutable $theft_vulnerability_start
 * @property string|CarbonImmutable $theft_vulnerability_end
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Solarsystem $solarsystem
 * @property-read Celestial|null $planet
 */
final class RaidableSkyhook extends Model
{
    public $incrementing = false;

    protected $primaryKey = 'planet_id';

    protected $keyType = 'int';

    /**
     * The solar system this skyhook is located in.
     *
     * @return BelongsTo<Solarsystem, $this>
     */
    public function solarsystem(): BelongsTo
    {
        return $this->belongsTo(Solarsystem::class);
    }

    /**
     * The planet (celestial) this skyhook orbits.
     *
     * @return BelongsTo<Celestial, $this>
     */
    public function planet(): BelongsTo
    {
        return $this->belongsTo(Celestial::class, 'planet_id');
    }

    protected function casts(): array
    {
        return [
            'planet_id' => 'integer',
            'solarsystem_id' => 'integer',
            'theft_vulnerability_start' => 'datetime',
            'theft_vulnerability_end' => 'datetime',
        ];
    }
}
