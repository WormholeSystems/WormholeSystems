<?php

namespace App\Http\Requests;

use App\Enums\SignatureCategory;
use App\Models\MapSolarsystem;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;
use Illuminate\Validation\Rule;

class PasteSignaturesRequest extends FormRequest
{
    public MapSolarsystem $map_solarsystem {
        get {
            return MapSolarsystem::query()->findOrFail($this->integer('map_solarsystem_id'));
        }
    }

    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(#[CurrentUser] User $user): bool
    {
        return $user->can('update', $this->map_solarsystem);
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(): array
    {
        return [
            'map_solarsystem_id' => ['required', 'integer', 'exists:map_solarsystems,id'],
            'signatures' => ['required', 'array'],
            'signatures.*.signature_id' => ['required', 'string', 'max:7', 'min:7'],
            'signatures.*.category' => ['nullable', 'sometimes', Rule::enum(SignatureCategory::class)],
            'signatures.*.type' => ['nullable', 'sometimes', 'string', 'max:255'],
        ];
    }
}
