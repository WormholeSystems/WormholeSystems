<?php

declare(strict_types=1);

namespace App\Casts;

use App\Enums\SolarsystemClass;
use Illuminate\Contracts\Database\Eloquent\CastsAttributes;
use Illuminate\Database\Eloquent\Model;

/**
 * @implements CastsAttributes<array<SolarsystemClass>, array<string>>
 */
final class SolarsystemClassArrayCast implements CastsAttributes
{
    /**
     * Cast the given value.
     *
     * @param  array<array-key, mixed>  $attributes
     * @return array<SolarsystemClass>|null
     */
    public function get(Model $model, string $key, mixed $value, array $attributes): ?array
    {
        if ($value === null) {
            return null;
        }

        $decoded = json_decode((string) $value, true);

        if (! is_array($decoded)) {
            return null;
        }

        return array_map(
            SolarsystemClass::from(...),
            $decoded
        );
    }

    /**
     * Prepare the given value for storage.
     *
     * @param  array<array-key, mixed>  $attributes
     * @param  array<SolarsystemClass|string>|null  $value
     */
    public function set(Model $model, string $key, mixed $value, array $attributes): ?string
    {
        if ($value === null) {
            return null;
        }

        $values = array_map(
            fn (SolarsystemClass|string $class) => $class instanceof SolarsystemClass ? $class->value : $class,
            $value
        );

        return json_encode($values);
    }
}
