<?php

declare(strict_types=1);

namespace App\Http\Requests;

use App\Models\MapRouteSolarsystem;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Container\Attributes\RouteParameter;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

final class UpdateMapRouteSolarsystemRequest extends FormRequest
{
    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(#[CurrentUser] User $user, #[RouteParameter('map_route_solarsystem')] MapRouteSolarsystem $mapRouteSolarsystem): bool
    {
        return $user->can('update', $mapRouteSolarsystem);
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(): array
    {
        return [
            'is_pinned' => ['nullable', 'boolean'],
        ];
    }
}
