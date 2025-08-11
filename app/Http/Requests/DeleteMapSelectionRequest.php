<?php

declare(strict_types=1);

namespace App\Http\Requests;

use App\Enums\Permission;
use App\Models\Map;
use App\Models\User;
use App\Rules\NotPinned;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Foundation\Http\FormRequest;

final class DeleteMapSelectionRequest extends FormRequest
{
    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(#[CurrentUser] User $user): bool
    {
        return Map::query()
            ->whereHas('mapSolarsystems', fn (Builder $query) => $query->whereIn('id', $this->array('map_solarsystem_ids')))
            ->whereDoesntHave('mapAccessors', fn (Builder $query) => $query->whereIn('accessible_id', $user->getAccessibleIds())->where('permission', Permission::Write))
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
            'map_solarsystem_ids' => ['required', 'array'],
            'map_solarsystem_ids.*' => ['required', 'integer', 'exists:map_solarsystems,id', new NotPinned],
        ];
    }
}
