<?php

declare(strict_types=1);

namespace App\Models;

use Carbon\CarbonImmutable;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use NicolasKion\SDE\ClassResolver;

/**
 * TypeAttribute model representing a type attribute in the game.
 *
 * @property int $id
 * @property int $type_id
 * @property int $attribute_id
 * @property float $value
 * @property-read string|CarbonImmutable $created_at
 * @property-read string|CarbonImmutable $updated_at
 * @property-read Type $type
 * @property-read Attribute $attribute
 */
final class TypeAttribute extends Model
{
    public $incrementing = false;

    /**
     * @return BelongsTo<Type,$this>
     */
    public function type(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::type());
    }

    /**
     * @return BelongsTo<Attribute,$this>
     */
    public function attribute(): BelongsTo
    {
        return $this->belongsTo(ClassResolver::attribute());
    }

    protected function casts(): array
    {
        return [
            'value' => 'float',
        ];
    }
}
