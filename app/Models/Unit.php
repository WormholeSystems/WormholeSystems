<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Collection;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\HasMany;
use NicolasKion\SDE\ClassResolver;

/**
 * Unit model representing a unit in the game.
 *
 * @property int $id
 * @property string $name
 * @property string $display_name
 * @property string|null $description
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Collection<int,Attribute> $attributes
 */
class Unit extends Model
{
    public $incrementing = false;

    /**
     * @return HasMany<Attribute, $this>
     */
    public function attributes(): HasMany
    {
        return $this->hasMany(ClassResolver::attribute(), 'unit_id');
    }
}
