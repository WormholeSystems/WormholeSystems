<?php

namespace App\Http\Requests;

use App\Models\Map;
use App\Models\MapSolarsystem;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

class CreateStatisticsRequest extends FormRequest
{
    public Map $map {
        get {
            return Map::query()->findOrFail($this->input('map_id'));
        }
    }

    public ?MapSolarsystem $mapSolarsystem {
        get {
            return $this->input('map_solarsystem_id')
                ? MapSolarsystem::query()->findOrFail($this->input('map_solarsystem_id'))
                : null;
        }
    }

    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(#[CurrentUser] User $user): bool
    {
        if ($this->mapSolarsystem) {
            return $user->can('update', $this->mapSolarsystem);
        }

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
            'map_id' => ['required', 'integer', 'exists:maps,id'],
            'map_solarsystem_id' => ['nullable', 'sometimes', 'integer', 'exists:map_solarsystems,id'],
            'top' => ['nullable', 'sometimes', 'integer', 'min:0', 'max:10'],
            'active_threshold' => ['nullable', 'sometimes', 'integer', 'min:0', 'max:1000'],
            'hostile_threshold' => ['nullable', 'sometimes', 'integer', 'min:0', 'max:1000'],
            'days_active' => ['nullable', 'sometimes', 'integer', 'min:0', 'max:365'],
            'days' => ['nullable', 'sometimes', 'integer', 'min:0', 'max:365'],
        ];
    }
}
