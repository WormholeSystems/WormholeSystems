<?php

declare(strict_types=1);

namespace App\Http\Requests;

use App\Enums\KillmailFilter;
use App\Enums\MassStatus;
use App\Enums\RoutePreference;
use App\Models\MapUserSetting;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Container\Attributes\RouteParameter;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;
use Illuminate\Validation\Rule;

final class UpdateMapUserSettingRequest extends FormRequest
{
    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(#[CurrentUser] User $user, #[RouteParameter('map_user_setting')] MapUserSetting $mapUserSetting): bool
    {
        return $user->can('update', $mapUserSetting);
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(): array
    {
        return [
            'tracking_allowed' => ['boolean'],
            'is_tracking' => ['boolean'],
            'route_allow_eol' => ['boolean'],
            'route_allow_mass_status' => ['nullable', 'string', Rule::enum(MassStatus::class)],
            'route_use_evescout' => ['boolean'],
            'route_preference' => ['nullable', 'string', Rule::enum(RoutePreference::class)],
            'security_penalty' => ['nullable', 'integer', 'min:0', 'max:100'],
            'killmail_filter' => ['nullable', 'string', Rule::enum(KillmailFilter::class)],
            'introduction_confirmed_at' => ['nullable', 'string', 'date'],
        ];
    }
}
