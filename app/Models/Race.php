<?php

declare(strict_types=1);

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\HasMany;
use NicolasKion\SDE\ClassResolver;

class Race extends Model
{
    public $incrementing = false;

    protected $fillable = [
        'id',
        'name',
        'description',
        'short_description',
        'icon_id',
    ];

    /**
     * @return HasMany<Type,$this>
     */
    public function types(): HasMany
    {
        return $this->hasMany(ClassResolver::type(), 'race_id');
    }

    /**
     * @return HasMany<Character,$this>
     */
    public function characters(): HasMany
    {
        return $this->hasMany(ClassResolver::character(), 'race_id');
    }
}
