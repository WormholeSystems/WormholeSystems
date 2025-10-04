<?php

declare(strict_types=1);

namespace App\Http\Requests\Signatures;

use App\Enums\LifetimeStatus;
use App\Enums\MassStatus;
use App\Models\MapConnection;
use App\Models\MapSolarsystem;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;
use Illuminate\Validation\Rule;

final class StoreSignatureRequest extends FormRequest
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

        if (! $this->mapConnection instanceof MapConnection) {
            return true;
        }
        if ($this->mapConnection->fromMapSolarsystem()->is($this->mapSolarsystem)) {
            return true;
        }

        return $this->mapConnection->toMapSolarsystem()->is($this->mapSolarsystem);
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
            'signature_category_id' => ['nullable', 'sometimes', 'integer', 'exists:signature_categories,id'],
            'signature_type_id' => ['nullable', 'sometimes', 'integer', 'exists:signature_types,id'],
            'map_connection_id' => ['nullable', 'sometimes', 'integer', 'exists:map_connections,id'],
            'lifetime' => ['nullable', 'sometimes', Rule::enum(LifetimeStatus::class)],
            'mass_status' => ['nullable', 'sometimes', Rule::enum(MassStatus::class)],
        ];
    }
}
