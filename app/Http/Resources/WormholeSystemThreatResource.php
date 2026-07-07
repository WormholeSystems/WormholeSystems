<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\WormholeSystemThreat;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin WormholeSystemThreat
 */
final class WormholeSystemThreatResource extends JsonResource
{
    /**
     * @return array<string, mixed>
     */
    public function toArray(Request $request): array
    {
        return [
            'id' => $this->entity_id,
            'name' => $this->name,
            'type' => $this->entity_type,
            'kills' => $this->kills,
        ];
    }
}
