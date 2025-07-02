<?php

namespace App\Http\Requests;

use App\Models\MapSolarsystem;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

class StoreMapConnectionRequest extends FormRequest
{
    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(): bool
    {
        return true;
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(): array
    {
        return [
            'from_map_solarsystem_id' => ['required', 'exists:map_solarsystems,id'],
            'to_map_solarsystem_id' => ['required', 'exists:map_solarsystems,id', 'different:from_map_solarsystem_id'],
        ];
    }

    public MapSolarsystem $fromMapSolarsystem
        {
        get {
            return MapSolarsystem::findOrFail($this->input('from_map_solarsystem_id'));
        }
    }

    public MapSolarsystem $toMapSolarsystem
        {
        get {
            return MapSolarsystem::findOrFail($this->input('to_map_solarsystem_id'));
        }
    }
}
