<?php

namespace App\Http\Requests;

use App\Enums\Permission;
use App\Models\MapSolarsystem;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

class StoreTrackingRequest extends FormRequest
{
    public MapSolarsystem $solarsystem {
        get => MapSolarsystem::query()
            ->where('id', $this->integer('from_map_solarsystem_id'))
            ->firstOrFail();
    }

    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(#[CurrentUser] User $user): bool
    {
        return $this->solarsystem->map->mapAccessors()
            ->whereIn('accessible_id', $user->getAccessibleIds())
            ->where('permission', Permission::Write)
            ->exists();
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
