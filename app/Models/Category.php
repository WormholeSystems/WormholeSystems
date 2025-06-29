<?php

declare(strict_types=1);

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\HasMany;
use NicolasKion\SDE\ClassResolver;

class Category extends Model
{
    public $incrementing = false;

    protected $fillable = [
        'id',
        'name',
        'icon_id',
        'published',
    ];

    /**
     * @return HasMany<Group,$this>
     */
    public function groups(): HasMany
    {
        return $this->hasMany(ClassResolver::group(), 'category_id');
    }
}
