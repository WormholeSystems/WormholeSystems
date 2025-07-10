<?php

namespace App\Http\Requests;

use App\Enums\MapSolarsystemStatus;
use App\Models\MapSolarsystem;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Container\Attributes\RouteParameter;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;
use Illuminate\Validation\Rule;

class UpdateMapSolarsystemRequest extends FormRequest
{
    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(#[RouteParameter('map_solarsystem')] MapSolarsystem $mapSolarsystem, #[CurrentUser] User $user): bool
    {
        return $user->can('update', $mapSolarsystem);
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(): array
    {
        return [
            'alias' => ['nullable', 'string', 'max:255'],
            'occupier_alias' => ['nullable', 'string', 'max:255'],
            'position_x' => ['nullable', 'numeric'],
            'position_y' => ['nullable', 'numeric'],
            'status' => ['nullable', 'sometimes', Rule::enum(MapSolarsystemStatus::class)],
            'pinned' => ['boolean'],
        ];
    }
}
