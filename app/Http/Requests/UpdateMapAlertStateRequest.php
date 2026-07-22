<?php

declare(strict_types=1);

namespace App\Http\Requests;

use App\Models\MapAlert;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

final class UpdateMapAlertStateRequest extends FormRequest
{
    public MapAlert $mapAlert {
        get => $this->route('map_alert');
    }

    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(#[CurrentUser] User $user): bool
    {
        return $user->can('update', $this->mapAlert);
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(): array
    {
        return ['enabled' => ['required', 'boolean']];
    }

    /** @return array<string, string> */
    public function messages(): array
    {
        return ['enabled.required' => 'Choose whether the alert should be enabled or disabled.'];
    }
}
