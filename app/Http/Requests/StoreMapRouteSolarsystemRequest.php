<?php

declare(strict_types=1);

namespace App\Http\Requests;

use App\Models\Map;
use App\Models\MapRouteSolarsystem;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

final class StoreMapRouteSolarsystemRequest extends FormRequest
{
    public Map $map {
        get => Map::query()->findOrFail(
            $this->integer('map_id'),
        );
    }

    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(#[CurrentUser] User $user): bool
    {
        return $user->can('create', [MapRouteSolarsystem::class, $this->map]);
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
            'solarsystem_id' => ['required', 'integer', 'exists:solarsystems,id'],
            'is_pinned' => ['nullable', 'boolean'],
        ];
    }
}
