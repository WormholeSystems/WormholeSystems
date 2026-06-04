<?php

declare(strict_types=1);

namespace App\Http\Requests;

use App\Http\Requests\Concerns\HasMapWebhookRules;
use App\Models\Map;
use App\Models\MapWebhook;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

final class StoreMapWebhookRequest extends FormRequest
{
    use HasMapWebhookRules;

    public Map $map {
        get => Map::query()->findOrFail(
            $this->integer('map_id'),
        );
    }

    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(#[CurrentUser] User $user): bool
    {
        return $user->can('create', [MapWebhook::class, $this->map]);
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, ValidationRule|array|string>
     */
    public function rules(): array
    {
        return [
            'map_id' => ['required', 'integer', 'exists:maps,id'],
            'discord_webhook_url' => ['required', 'url', 'regex:#^https://discord(app)?\.com/api/webhooks/#'],
            ...$this->webhookRules(),
        ];
    }
}
