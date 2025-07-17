<?php

namespace App\Http\Requests\Signatures;

use App\Enums\SignatureCategory;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;
use Illuminate\Validation\Rule;

class StoreSignatureRequest extends FormRequest
{
    public MapSolarsystem $mapSolarsystem {
        get {
            return MapSolarsystem::query()->findOrFail($this->integer('map_solarsystem_id'));
        }
    }

    public ?MapConnection $mapConnection {
        get {
            if (! $this->has('map_connection_id')) {
                return null;
            }

            return MapConnection::query()->find($this->integer('map_connection_id'));
        }
    }

    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(#[CurrentUser] User $user): bool
    {
        if (! $user->can('update', $this->mapSolarsystem)) {
            return false;
        }

        if (! $this->mapConnection) {
            return true;
        }

        return $this->mapConnection->fromMapSolarsystem()->is($this->mapSolarsystem)
            || $this->mapConnection->toMapSolarsystem()->is($this->mapSolarsystem);
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
            'signature_id' => ['nullable', 'sometimes', 'string', 'max:7', 'min:7'],
            'category' => ['nullable', 'sometimes', Rule::enum(SignatureCategory::class)],
            'type' => ['nullable', 'sometimes', 'string', 'max:255'],
            'map_connection_id' => ['nullable', 'sometimes', 'integer', 'exists:map_connections,id'],
        ];
    }
}
