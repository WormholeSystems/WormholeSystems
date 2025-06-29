<?php

declare(strict_types=1);

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\HasMany;
use NicolasKion\SDE\ClassResolver;

class Region extends Model
{
    public $incrementing = false;

    protected $fillable = [
        'id',
        'name',
        'type',
    ];

    /**
     * @return HasMany<Constellation,$this>
     */
    public function constellations(): HasMany
    {
        return $this->hasMany(ClassResolver::constellation(), 'region_id');
    }

    /**
     * @return HasMany<Solarsystem,$this>
     */
    public function solarsystems(): HasMany
    {
        return $this->hasMany(ClassResolver::solarsystem(), 'region_id');
    }

    /**
     * @return HasMany<Celestial,$this>
     */
    public function celestials(): HasMany
    {
        return $this->hasMany(ClassResolver::celestial(), 'region_id');
    }

    /**
     * @return HasMany<Station,$this>
     */
    public function stations(): HasMany
    {
        return $this->hasMany(ClassResolver::station(), 'region_id');
    }
}
