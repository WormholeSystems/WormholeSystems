<?php

declare(strict_types=1);

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Support\Facades\DB;
use NicolasKion\SDE\ClassResolver;
use Throwable;

class Character extends Model
{
    public $incrementing = false;

    protected $fillable = [
        'id',
        'name',
        'description',
        'race_id',
        'bloodline_id',
        'corporation_id',
        'faction_id',
        'alliance_id',
        'security_status',
        'gender',
        'birthday',
        'title',
        'user_id',
    ];

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
}
