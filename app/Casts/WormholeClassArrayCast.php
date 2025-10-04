<?php

declare(strict_types=1);

namespace App\Casts;

use App\Enums\WormholeClass;
use Illuminate\Contracts\Database\Eloquent\CastsAttributes;
use Illuminate\Database\Eloquent\Model;

/**
 * @implements CastsAttributes<array<WormholeClass>, array<string>>
 */
class WormholeClassArrayCast implements CastsAttributes
{
    /**
     * Cast the given value.
     *
     * @param  array<array-key, mixed>  $attributes
     * @return array<WormholeClass>|null
     */
    public function get(Model $model, string $key, mixed $value, array $attributes): ?array
    {
        if ($value === null) {
            return null;
        }

        $decoded = json_decode($value, true);

        if (! is_array($decoded)) {
            return null;
        }

        return array_map(
            fn (string $class) => WormholeClass::from($class),
            $decoded
        );
    }

    /**
     * Prepare the given value for storage.
     *
     * @param  array<array-key, mixed>  $attributes
     * @param  array<WormholeClass|string>|null  $value
     */
    public function set(Model $model, string $key, mixed $value, array $attributes): ?string
    {
        if ($value === null) {
            return null;
        }

        $values = array_map(
            fn (WormholeClass|string $class) => $class instanceof WormholeClass ? $class->value : $class,
            $value
        );

        return json_encode($values);
    }
}


