<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Enums\MapAlertDeliveryType;
use App\Models\MapAlert;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin MapAlert
 */
final class MapAlertResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     * @return array<string, mixed>
     */
    public function toArray(Request $request): array
    {
        return [
            'id' => $this->id,
            'map' => $this->whenLoaded('map', fn (): array => [
                'id' => $this->map->id,
                'name' => $this->map->name,
                'slug' => $this->map->slug,
            ]),
            'creator_name' => $this->whenLoaded(
                'creator',
                fn (): ?string => $this->creator?->alertDisplayName(),
            ),
            'delivery_type' => $this->delivery_type->value,
            'destination_summary' => $this->delivery_type === MapAlertDeliveryType::Webhook
                ? $this->whenLoaded('webhook', fn (): string => $this->webhook->name)
                : $this->delivery_type->label(),
            'map_webhook_id' => $this->map_webhook_id,
            'map_webhook_role_id' => $this->map_webhook_role_id,
            'mention_mode' => $this->mention_mode->value,
            'mention_mode_label' => $this->mention_mode->label(),
            'type' => $this->type->value,
            'target_solarsystem_id' => $this->target_solarsystem_id,
            'target_solarsystem' => $this->whenLoaded('targetSolarsystem', fn (): ?array => $this->targetSolarsystem ? [
                'id' => $this->targetSolarsystem->id,
                'name' => $this->targetSolarsystem->name,
            ] : null),
            'origin_solarsystem_id' => $this->origin_solarsystem_id,
            'origin_solarsystem' => $this->whenLoaded('originSolarsystem', fn (): ?array => $this->originSolarsystem ? [
                'id' => $this->originSolarsystem->id,
                'name' => $this->originSolarsystem->name,
            ] : null),
            'ship_type' => $this->ship_type?->value,
            'jdc_level' => $this->jdc_level,
            'include_highsec' => $this->include_highsec,
            'max_jumps' => $this->max_jumps,
            'filter_match' => $this->filter_match->value,
            'filters' => $this->filters->map->toArray()->all(),
            'is_active' => $this->is_active,
            'disabled_reason' => $this->disabled_reason?->value,
            'disabled_reason_label' => $this->disabled_reason?->label(),
            'disabled_at' => $this->disabled_at?->toIso8601String(),
            'last_fired_at' => $this->last_fired_at?->toIso8601String(),
            'created_at' => $this->created_at->toIso8601String(),
        ];
    }
}
