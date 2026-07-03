<?php

declare(strict_types=1);

namespace App\Http\Requests;

use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;
use Illuminate\Validation\Rule;

final class EveSearchRequest extends FormRequest
{
    /**
     * The route is already behind the `auth` middleware; any signed-in user may search
     * the static EVE reference data.
     */
    public function authorize(): bool
    {
        return true;
    }

    /**
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(): array
    {
        return [
            'kind' => ['required', Rule::in(['type', 'group'])],
            'q' => ['nullable', 'string', 'max:100'],
            'ids' => ['nullable', 'array'],
            'ids.*' => ['integer'],
            'category_id' => ['nullable', 'integer'],
        ];
    }
}
