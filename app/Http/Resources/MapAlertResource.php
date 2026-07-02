<?php

declare(strict_types=1);

namespace App\Http\Resources;

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
            'map_webhook_id' => $this->map_webhook_id,
            'map_webhook_role_id' => $this->map_webhook_role_id,
            'type' => $this->type->value,
            'target_solarsystem_id' => $this->target_solarsystem_id,
            'ship_type' => $this->ship_type?->value,
            'jdc_level' => $this->jdc_level,
            'include_highsec' => $this->include_highsec,
            'max_jumps' => $this->max_jumps,
            'filter_match' => $this->filter_match->value,
            'filters' => $this->filters->map->toArray()->all(),
            'is_active' => $this->is_active,
            'last_fired_at' => $this->last_fired_at?->toIso8601String(),
        ];
    }
}
