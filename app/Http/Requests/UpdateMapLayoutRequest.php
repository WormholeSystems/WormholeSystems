<?php

declare(strict_types=1);

namespace App\Http\Requests;

use App\Enums\MapLayout;
use App\Models\Map;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Container\Attributes\RouteParameter;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;
use Illuminate\Validation\Rule;

final class UpdateMapLayoutRequest extends FormRequest
{
    /**
     * The layout mode affects every viewer, so only managers may change it.
     */
    public function authorize(#[RouteParameter('map')] Map $map, #[CurrentUser] User $user): bool
    {
        return $user->can('updateSettings', $map);
    }

    /**
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(): array
    {
        return [
            'layout' => ['required', Rule::enum(MapLayout::class)],
        ];
    }
}
