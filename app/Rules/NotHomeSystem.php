<?php

declare(strict_types=1);

namespace App\Rules;

use App\Models\MapSolarsystem;
use Closure;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Translation\PotentiallyTranslatedString;

final class NotHomeSystem implements ValidationRule
{
    /**
     * Run the validation rule.
     *
     * @param  Closure(string, ?string=): PotentiallyTranslatedString  $fail
     */
    public function validate(string $attribute, mixed $value, Closure $fail): void
    {
        $map_solarsystem = MapSolarsystem::query()->with('map')->find($value);

        if ($map_solarsystem !== null && $map_solarsystem->map->home_solarsystem_id === $map_solarsystem->solarsystem_id) {
            $fail('The selected solarsystem is the home system and cannot be deleted.');
        }
    }
}
