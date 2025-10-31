<?php

declare(strict_types=1);

namespace App\Http\Requests;

use App\Models\Map;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Foundation\Http\FormRequest;

final class StoreEveScoutConnectionRequest extends FormRequest
{
    public Map $map {
        get => Map::findOrFail($this->input('map_id'));
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
     * @return array<string, \Illuminate\Contracts\Validation\ValidationRule|array<mixed>|string>
     */
    public function rules(): array
    {
        return [
            'map_id' => ['required', 'integer', 'exists:maps,id'],
            'special_system_id' => ['required', 'integer', 'exists:solarsystems,id'],
        ];
    }
}
