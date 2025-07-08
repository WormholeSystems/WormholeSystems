<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Support\Facades\DB;
use NicolasKion\SDE\ClassResolver;
use Throwable;

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
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Race $race
 * @property-read Bloodline $bloodline
 * @property-read Corporation $corporation
 * @property-read Faction|null $faction
 * @property-read Alliance|null $alliance
 */
class Character extends Model
{
    public $incrementing = false;

    /**
     * @param  int[]  $ids
     *
     * @throws Throwable
     */
    public static function createFromIds(array $ids): void
    {
        DB::transaction(fn () => self::query()->upsert(
            collect($ids)->map(fn ($id) => ['id' => $id])->toArray(),
            ['id']
        ), 5);
    }

    /**
     * @return BelongsTo<Race,$this>
     */
    public function race(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::race());
    }

    /**
     * @return BelongsTo<Bloodline,$this>
     */
    public function bloodline(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::bloodline());
    }

    /**
     * @return BelongsTo<Corporation,$this>
     */
    public function corporation(): BelongsTo
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
     * @return BelongsTo<Alliance,$this>
     */
    public function alliance(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::alliance());
    }

    protected function casts(): array
    {
        return [
            'birthday' => 'immutable_datetime',
        ];
    }
}
