<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\Character;
use App\Models\Corporation;
use App\Models\MapAccess;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin MapAccess
 */
final class MapAccessEntityResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     * @return array<string, mixed>
     */
    public function toArray(Request $request): array
    {
        return [
            'id' => $this->accessible_id,
            'name' => $this->accessible->name,
            'type' => match ($this->accessible_type) {
                Character::class => 'character',
                Corporation::class => 'corporation',
                default => 'alliance',
            },
            'permission' => $this->permission->value,
            'is_owner' => $this->is_owner,
            'expires_at' => $this->expires_at?->toIso8601String(),
        ];
    }
}
