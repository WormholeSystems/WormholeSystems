<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\MapWebhookRole;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin MapWebhookRole
 */
final class MapWebhookRoleResource extends JsonResource
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
            'name' => $this->name,
            'discord_role_id' => $this->discord_role_id,
            'alerts_count' => $this->whenCounted('alerts'),
        ];
    }
}
