<?php

declare(strict_types=1);

namespace App\Rules;

use Closure;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Support\Facades\DB;
use Illuminate\Translation\PotentiallyTranslatedString;

final readonly class SameParent implements ValidationRule
{
    public function __construct(
        private string $item_table,
        private string $parent_column,
        private string $value_column = 'id',
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
            ->distinct($this->parent_column)
            ->count();

        if ($parents_count > 1) {
            $fail("The selected $attribute must have the same parent.");
        }
    }
}
