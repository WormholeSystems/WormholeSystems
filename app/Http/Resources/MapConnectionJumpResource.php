<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\MapConnectionJump;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * Kept flat on purpose: jumps ride along on every connection broadcast, so the
 * payload stays small enough for Reverb's message size limit.
 *
 * @mixin MapConnectionJump
 */
final class MapConnectionJumpResource extends JsonResource
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
            'character_id' => $this->character_id,
            'character_name' => $this->character?->name,
            'ship_type_id' => $this->ship_type_id,
            'ship_type_name' => $this->shipType?->name,
            'ship_name' => $this->ship_name,
            'mass' => $this->mass,
            'is_manual' => $this->is_manual,
            'from_solarsystem_id' => $this->from_solarsystem_id,
            'to_solarsystem_id' => $this->to_solarsystem_id,
            'created_at' => $this->created_at,
        ];
    }
}
