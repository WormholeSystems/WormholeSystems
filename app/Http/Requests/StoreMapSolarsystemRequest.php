<?php

namespace App\Http\Requests;

use App\Models\Map;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

class StoreMapSolarsystemRequest extends FormRequest
{
    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(#[CurrentUser] User $user): bool
    {
        return $user->can('update', $this->map);
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(): array
    {
        return [
            'map_id' => ['required', 'exists:maps,id'],
            'solarsystem_id' => ['required', 'exists:solarsystems,id'],
            'alias' => ['nullable', 'string', 'max:255'],
            'occupier_alias' => ['nullable', 'string', 'max:255'],
            'position_x' => ['nullable', 'numeric'],
            'position_y' => ['nullable', 'numeric'],
            'status' => ['nullable', 'sometimes', 'in:active,inactive'],
            'pinned' => ['boolean'],
        ];
    }

    public Map $map {
        get {
            return Map::findOrFail($this->input('map_id'));
        }
    }
}
