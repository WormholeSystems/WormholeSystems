<?php

namespace App\Http\Requests;

use App\Rules\NotPinned;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

class DeleteMapSelectionRequest extends FormRequest
{
    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(): bool
    {
        return true;
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(): array
    {
        return [
            'map_solarsystem_ids' => ['required', 'array'],
            'map_solarsystem_ids.*' => ['required', 'integer', 'exists:map_solarsystems,id', new NotPinned],
        ];
    }
}
