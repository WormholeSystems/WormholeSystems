<?php

declare(strict_types=1);

namespace App\Http\Requests;

use App\Models\MapWebhook;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

final class UpdateMapWebhookRequest extends FormRequest
{
    public MapWebhook $mapWebhook {
        get => $this->route('map_webhook');
    }

    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(#[CurrentUser] User $user): bool
    {
        return $user->can('update', $this->mapWebhook);
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * The Discord URL is optional on update so the secret never needs to round-trip
     * to the client; leaving it blank keeps the stored value.
     *
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(): array
    {
        return [
            'name' => ['required', 'string', 'max:255'],
            'discord_webhook_url' => ['nullable', 'url', 'regex:#^https://discord(app)?\.com/api/webhooks/#'],
            'target_solarsystem_id' => ['required', 'integer', 'exists:solarsystems,id'],
            'max_jumps' => ['required', 'integer', 'between:1,20'],
            'is_active' => ['boolean'],
        ];
    }
}
