<?php

declare(strict_types=1);

namespace App\Http\Requests;

use App\Models\MapWebhookRole;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

final class UpdateMapWebhookRoleRequest extends FormRequest
{
    public MapWebhookRole $mapWebhookRole {
        get => $this->route('map_webhook_role');
    }

    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(#[CurrentUser] User $user): bool
    {
        return $user->can('update', $this->mapWebhookRole);
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(): array
    {
        return [
            'name' => ['required', 'string', 'max:255'],
            'discord_role_id' => ['required', 'string', 'regex:/^\d+$/'],
        ];
    }
}
