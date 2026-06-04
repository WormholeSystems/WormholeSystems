<?php

declare(strict_types=1);

namespace App\Http\Requests;

use App\Http\Requests\Concerns\HasMapWebhookRules;
use App\Models\MapWebhook;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

final class UpdateMapWebhookRequest extends FormRequest
{
    use HasMapWebhookRules;

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
            'discord_webhook_url' => ['nullable', 'url', 'regex:#^https://discord(app)?\.com/api/webhooks/#'],
            ...$this->webhookRules(),
        ];
    }
}
