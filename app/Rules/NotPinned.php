<?php

namespace App\Rules;

use App\Models\MapSolarsystem;
use Closure;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Translation\PotentiallyTranslatedString;

class NotPinned implements ValidationRule
{
    /**
     * Run the validation rule.
     *
     * @param  Closure(string, ?string=): PotentiallyTranslatedString  $fail
     */
    public function validate(string $attribute, mixed $value, Closure $fail): void
    {
        if (MapSolarsystem::query()->whereId($value)->wherePinned(true)->exists()) {
            $fail('The selected solarsystem is pinned and cannot be deleted.');
        }
    }
}
