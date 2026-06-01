<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\MapWebhook;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin MapWebhook
 */
final class MapWebhookResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     * The Discord webhook URL is a credential and is never exposed to the client.
     *
     * @return array<string, mixed>
     */
    public function toArray(Request $request): array
    {
        return [
            'id' => $this->id,
            'name' => $this->name,
            'type' => $this->type->value,
            'target_solarsystem_id' => $this->target_solarsystem_id,
            'max_jumps' => $this->max_jumps,
            'is_active' => $this->is_active,
            'last_fired_at' => $this->last_fired_at?->toIso8601String(),
        ];
    }
}
