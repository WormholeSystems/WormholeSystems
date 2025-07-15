<?php

namespace App\Http\Requests;

use App\Models\MapUserSetting;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Container\Attributes\RouteParameter;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

class UpdateMapUserSettingRequest extends FormRequest
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
        ];
    }
}
