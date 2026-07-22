<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Enums\MapAlertDeliveryType;
use App\Models\MapAlertEvent;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin MapAlertEvent
 */
final class MapAlertEventResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     * @return array<string, mixed>
     */
    public function toArray(Request $request): array
    {
        $deliveryType = $this->snapshot['delivery_type'] ?? null;

        return [
            'id' => $this->id,
            'action' => $this->action->value,
            'reason' => $this->reason ?? ($this->snapshot['disabled_reason'] ?? null),
            'actor_name' => $this->actor?->alertDisplayName() ?? $this->actor_name,
            'delivery_type' => $deliveryType,
            'destination_summary' => MapAlertDeliveryType::tryFrom(is_string($deliveryType) ? $deliveryType : '')?->label() ?? 'Alert',
            'type' => $this->snapshot['type'] ?? null,
            'created_at' => $this->created_at->toIso8601String(),
        ];
    }
}
