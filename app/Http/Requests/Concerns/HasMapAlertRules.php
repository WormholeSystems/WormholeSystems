<?php

declare(strict_types=1);

namespace App\Http\Requests\Concerns;

use App\Enums\JumpShipType;
use App\Enums\KillmailFilterMatch;
use App\Enums\KillmailFilterMode;
use App\Enums\KillmailFilterSide;
use App\Enums\KillmailFilterSubject;
use App\Enums\MapAlertMentionMode;
use App\Enums\MapAlertType;
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
            'map_webhook_role_id' => ['nullable', 'integer', Rule::exists('map_webhook_roles', 'id')->where('map_id', $mapId), 'prohibited_if:mention_mode,'.MapAlertMentionMode::Everyone->value],
            'mention_mode' => ['sometimes', Rule::in([MapAlertMentionMode::None->value, MapAlertMentionMode::Everyone->value])],
            'type' => ['required', Rule::enum(MapAlertType::class)],
            'target_solarsystem_id' => ['nullable', 'integer', 'exists:solarsystems,id', 'required_if:type,'.MapAlertType::Proximity->value.','.MapAlertType::JumpRange->value],
            'origin_solarsystem_id' => ['nullable', 'integer', 'exists:solarsystems,id', 'prohibited_unless:type,'.MapAlertType::Proximity->value],
            'ship_type' => ['nullable', Rule::enum(JumpShipType::class), 'required_if:type,'.MapAlertType::JumpRange->value],
            'jdc_level' => ['nullable', 'integer', 'between:1,5', 'required_if:type,'.MapAlertType::JumpRange->value],
            'include_highsec' => ['boolean'],
            'max_jumps' => ['nullable', 'integer', 'between:1,20', 'required_unless:type,'.MapAlertType::JumpRange->value],
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
