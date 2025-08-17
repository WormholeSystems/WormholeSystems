<?php

declare(strict_types=1);

namespace App\Http\Requests;

use App\Enums\MapSolarsystemStatus;
use App\Models\MapSolarsystem;
use App\Models\User;
use Illuminate\Container\Attributes\Config;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Container\Attributes\RouteParameter;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;
use Illuminate\Validation\Rule;

final class UpdateMapSolarsystemRequest extends FormRequest
{
    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(#[RouteParameter('map_solarsystem')] MapSolarsystem $mapSolarsystem, #[CurrentUser] User $user): bool
    {
        return $user->can('update', $mapSolarsystem);
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(
        #[Config('map.grid_size')] float $grid_size,
        #[Config('map.max_size.x')] float $max_size_x,
        #[Config('map.max_size.y')] float $max_size_y
    ): array {
        return [
            'alias' => ['nullable', 'string', 'max:255'],
            'occupier_alias' => ['nullable', 'string', 'max:255'],
            'position_x' => ['nullable',  Rule::numeric()->min($grid_size)->max($max_size_x - $grid_size)],
            'position_y' => ['nullable', Rule::numeric()->min($grid_size)->max($max_size_y - $grid_size)],
            'status' => ['nullable', 'sometimes', Rule::enum(MapSolarsystemStatus::class)],
            'pinned' => ['boolean'],
            'notes' => ['nullable', 'string', 'max:65535'],
        ];
    }
}
