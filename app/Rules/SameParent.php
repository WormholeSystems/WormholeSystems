<?php

namespace App\Rules;

use Closure;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Support\Facades\DB;
use Illuminate\Translation\PotentiallyTranslatedString;

class SameParent implements ValidationRule
{
    public function __construct(
        private readonly string $item_table,
        private readonly string $parent_column,
        private readonly string $value_column = 'id',
    ) {}

    /**
     * Run the validation rule.
     *
     * @param  Closure(string, ?string=): PotentiallyTranslatedString  $fail
     */
    public function validate(string $attribute, mixed $value, Closure $fail): void
    {
        $items = collect($value);

        if ($items->isEmpty()) {
            return;
        }

        $parents_count = DB::table($this->item_table)
            ->whereIn($this->value_column, $items->pluck($this->value_column))
            ->select($this->parent_column)
            ->distinct()
            ->pluck($this->parent_column)
            ->count();

        if ($parents_count > 1) {
            $fail("The selected {$attribute} must have the same parent.");
        }
    }
}
