<?php

declare(strict_types=1);

namespace App\Http\Requests;

use App\Models\MapSolarsystem;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

final class PasteSignaturesRequest extends FormRequest
{
    public MapSolarsystem $map_solarsystem {
        get => MapSolarsystem::query()->findOrFail($this->integer('map_solarsystem_id'));
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
            'signatures.*.signature_category_id' => ['nullable', 'sometimes', 'integer', 'exists:signature_categories,id'],
            'signatures.*.signature_type_id' => ['nullable', 'sometimes', 'integer', 'exists:signature_types,id'],
        ];
    }
}
