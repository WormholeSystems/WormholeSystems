<?php

namespace App\Http\Requests;

use App\Models\Map;
use App\Models\User;
use App\Rules\SameParent;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

class UpdateMapSelectionRequest extends FormRequest
{
    public Map $map {
        get {
            return Map::query()
                ->whereHas('mapSolarsystems', fn ($query) => $query->whereIn('id', $this->array('map_solarsystems.*.id')))
                ->firstOrFail();
        }
    }

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
            'map_solarsystems' => ['required', 'array', new SameParent('map_solarsystems', 'map_id', 'id')],
            'map_solarsystems.*' => ['required', 'array'],
            'map_solarsystems.*.id' => ['required', 'integer', 'exists:map_solarsystems,id'],
            'map_solarsystems.*.position_x' => ['required', 'numeric', 'min:0', 'max:4000'],
            'map_solarsystems.*.position_y' => ['required', 'numeric', 'min:0', 'max:1600'],
        ];
    }
}
