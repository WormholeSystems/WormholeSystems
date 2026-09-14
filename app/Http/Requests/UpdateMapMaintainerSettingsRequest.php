<?php

declare(strict_types=1);

namespace App\Http\Requests;

use App\Models\Map;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Container\Attributes\RouteParameter;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

final class UpdateMapMaintainerSettingsRequest extends FormRequest
{
    /**
     * The maintainer leaderboard scoring is shared by every viewer, so only managers may
     * change it.
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
            'maintainer_points_created' => ['sometimes', 'integer', 'between:0,100'],
            'maintainer_points_updated' => ['sometimes', 'integer', 'between:0,100'],
            'maintainer_points_deleted' => ['sometimes', 'integer', 'between:0,100'],
            'maintainer_minimum_points' => ['sometimes', 'integer', 'between:0,10000'],
        ];
    }
}
