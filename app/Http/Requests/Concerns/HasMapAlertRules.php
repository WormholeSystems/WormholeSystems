<?php

declare(strict_types=1);

namespace App\Http\Requests\Concerns;

use App\Enums\KillmailFilterMatch;
use App\Enums\KillmailFilterMode;
use App\Enums\KillmailFilterSide;
use App\Enums\KillmailFilterSubject;
use App\Enums\MapWebhookType;
use App\Models\Map;
use Illuminate\Validation\Rule;

trait HasMapAlertRules
{
    /**
     * The map an alert belongs to, used to scope the webhook and role it references.
     */
    abstract protected function alertMap(): Map;

    /**
     * Validation rules shared by storing and updating an alert. The store request adds
     * its own `map_id` rule around these.
     *
     * @return array<string, array<int, mixed>>
     */
    protected function alertRules(): array
    {
        $mapId = $this->alertMap()->id;

        return [
            'map_webhook_id' => ['required', 'integer', Rule::exists('map_webhooks', 'id')->where('map_id', $mapId)],
            'map_webhook_role_id' => ['nullable', 'integer', Rule::exists('map_webhook_roles', 'id')->where('map_id', $mapId)],
            'type' => ['required', Rule::enum(MapWebhookType::class)],
            'target_solarsystem_id' => ['nullable', 'integer', 'exists:solarsystems,id', 'required_if:type,'.MapWebhookType::Proximity->value],
            'max_jumps' => ['required', 'integer', 'between:1,20'],
            'filter_match' => ['nullable', Rule::enum(KillmailFilterMatch::class)],
            'filters' => ['nullable', 'array'],
            'filters.*.subject' => ['required', Rule::enum(KillmailFilterSubject::class)],
            'filters.*.side' => ['required', Rule::enum(KillmailFilterSide::class)],
            'filters.*.mode' => ['required', Rule::enum(KillmailFilterMode::class)],
            'filters.*.ids' => ['required', 'array', 'min:1'],
            'filters.*.ids.*' => ['required', 'integer', 'min:1'],
            'is_active' => ['boolean'],
        ];
    }
}
