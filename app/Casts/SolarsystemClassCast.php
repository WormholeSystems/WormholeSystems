<?php

declare(strict_types=1);

namespace App\Casts;

use App\Enums\SolarsystemClass;
use Illuminate\Contracts\Database\Eloquent\CastsAttributes;
use Illuminate\Database\Eloquent\Model;

/**
 * Casts a scalar class column to the SolarsystemClass enum.
 *
 * The underlying column is a numeric integer (e.g. 13) while the enum is
 * string-backed (e.g. '13'), so the value is coerced to a string before
 * resolving. The built-in enum cast would call from(13) and throw.
 *
 * @implements CastsAttributes<SolarsystemClass, SolarsystemClass|string|int>
 */
final class SolarsystemClassCast implements CastsAttributes
{
    /**
     * @param  array<array-key, mixed>  $attributes
     */
    public function get(Model $model, string $key, mixed $value, array $attributes): ?SolarsystemClass
    {
        if ($value === null) {
            return null;
        }

        return SolarsystemClass::from((string) $value);
    }

    /**
     * @param  array<array-key, mixed>  $attributes
     */
    public function set(Model $model, string $key, mixed $value, array $attributes): ?string
    {
        if ($value === null) {
            return null;
        }

        return $value instanceof SolarsystemClass ? $value->value : (string) $value;
    }
}
