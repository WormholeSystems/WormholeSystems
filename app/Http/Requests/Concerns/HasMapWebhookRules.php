<?php

declare(strict_types=1);

namespace App\Http\Requests\Concerns;

use App\Enums\KillmailFilterMatch;
use App\Enums\KillmailFilterMode;
use App\Enums\KillmailFilterSide;
use App\Enums\KillmailFilterSubject;
use App\Enums\MapWebhookType;
use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Validation\Rule;

trait HasMapWebhookRules
{
    /**
     * Treat a blank role id as "no ping" so any client can clear it, not just the
     * frontend that already trims empty input to null.
     */
    protected function prepareForValidation(): void
    {
        if ($this->has('discord_role_id') && blank($this->input('discord_role_id'))) {
            $this->merge(['discord_role_id' => null]);
        }
    }

    /**
     * Validation rules shared by storing and updating a map webhook. The store/update
     * requests add their own `map_id` / `discord_webhook_url` rules around these.
     *
     * @return array<string, array<int, ValidationRule|string>>
     */
    protected function webhookRules(): array
    {
        return [
            'name' => ['required', 'string', 'max:255'],
            'discord_role_id' => ['nullable', 'string', 'regex:/^\d+$/'],
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
