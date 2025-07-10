<?php

namespace App\Http\Requests;

use App\Enums\Permission;
use App\Models\Map;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

class StoreTrackingRequest extends FormRequest
{
    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(#[CurrentUser] User $user): bool
    {
        return Map::query()
            ->whereHas('mapSolarsystems', fn ($query) => $query->where('id', $this->integer('from_map_solarsystem_id')))
            ->whereDoesntHave('mapAccessors', fn ($query) => $query->where('accessible_id', $user->getAccessibleIds())->where('permission', Permission::Write))
            ->doesntExist();
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(): array
    {
        return [
            'from_map_solarsystem_id' => ['required', 'integer', 'exists:map_solarsystems,id'],
            'to_solarsystem_id' => ['required', 'integer', 'exists:solarsystems,id'],
        ];
    }
}
