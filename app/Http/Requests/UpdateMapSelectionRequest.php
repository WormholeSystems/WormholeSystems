<?php

namespace App\Http\Requests;

use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

class UpdateMapSelectionRequest extends FormRequest
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
            'map_solarsystems' => ['required', 'array'],
            'map_solarsystems.*' => ['required', 'array'],
            'map_solarsystems.*.id' => ['required', 'integer', 'exists:map_solarsystems,id'],
            'map_solarsystems.*.position_x' => ['required', 'numeric', 'min:0', 'max:4000'],
            'map_solarsystems.*.position_y' => ['required', 'numeric', 'min:0', 'max:1600'],
        ];
    }
}
