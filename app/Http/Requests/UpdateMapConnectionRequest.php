<?php

namespace App\Http\Requests;

use App\Enums\MassStatus;
use App\Enums\ShipSize;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;
use Illuminate\Validation\Rule;

class UpdateMapConnectionRequest extends FormRequest
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
            'wormhole_id' => ['nullable', 'sometimes', 'integer', 'exists:wormholes,id'],
            'mass_status' => ['nullable', 'sometimes', Rule::enum(MassStatus::class)],
            'ship_size' => ['nullable', 'sometimes', Rule::enum(ShipSize::class)],
            'is_eol' => ['nullable', 'sometimes', 'boolean'],
        ];
    }
}
