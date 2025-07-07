<?php

declare(strict_types=1);

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use NicolasKion\SDE\ClassResolver;

class Corporation extends Model
{
    public $incrementing = false;

    protected $fillable = [
        'id',
        'name',
        'ceo_id',
        'creator_id',
        'faction_id',
        'home_station_id',
        'member_count',
        'shares',
        'date_founded',
        'description',
        'url',
        'ticker',
        'tax_rate',
        'war_eligible',
        'npc',
        'alliance_id',
    ];

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

    protected function casts(): array
    {
        return [
            'date_founded' => 'datetime',
            'war_eligible' => 'boolean',
            'npc' => 'boolean',
        ];
    }
}
