<?php

declare(strict_types=1);

namespace App\Http\Requests;

use App\Enums\LifetimeStatus;
use App\Enums\MassStatus;
use App\Enums\Permission;
use App\Enums\ShipSize;
use App\Models\Map;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;
use Illuminate\Validation\Rule;

final class StoreMapConnectionRequest extends FormRequest
{
    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(#[CurrentUser] User $user): bool
    {
        return Map::query()
            ->whereHas('mapSolarsystems', fn ($query) => $query->whereIn('id', [$this->integer('from_map_solarsystem_id'), $this->integer('to_map_solarsystem_id')]))
            ->whereDoesntHave('mapAccessors', fn ($query) => $query->notExpired()->whereIn('accessible_id', $user->getAccessibleIds())->whereIn('permission', [Permission::Member, Permission::Manager]))
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
            'from_map_solarsystem_id' => ['required', 'exists:map_solarsystems,id'],
            'to_map_solarsystem_id' => ['required', 'exists:map_solarsystems,id', 'different:from_map_solarsystem_id'],
            'wormhole_id' => ['nullable', 'sometimes', 'integer', 'exists:wormholes,id'],
            'mass_status' => ['nullable', 'sometimes', 'string', Rule::enum(MassStatus::class)],
            'ship_size' => ['nullable', 'sometimes', 'string', Rule::enum(ShipSize::class)],
            'lifetime' => ['nullable', 'sometimes', Rule::enum(LifetimeStatus::class)],
        ];
    }
}
