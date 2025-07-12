<?php

namespace App\Http\Requests;

use App\Models\Character;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

class SetWaypointRequest extends FormRequest
{
    public Character $character {
        get {
            return Character::findOrFail($this->integer('character_id'));
        }
    }

    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(#[CurrentUser] User $user): bool
    {
        return $this->character->user()->is($user);
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(): array
    {
        return [
            'character_id' => 'required|integer|exists:characters,id',
            'destination_id' => 'required|integer|exists:solarsystems,id',
            'clear_other_waypoints' => 'sometimes|boolean',
            'add_to_beginning' => 'sometimes|boolean',
        ];
    }
}
