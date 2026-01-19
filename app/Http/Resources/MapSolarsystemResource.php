<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\MapSolarsystem;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Throwable;

/**
 * @mixin MapSolarsystem
 */
final class MapSolarsystemResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     * @return array<string, mixed>
     *
     * @throws Throwable
     */
    public function toArray(Request $request): array
    {
        return [
            'id' => $this->id,
            'map_id' => $this->map_id,
            'alias' => $this->alias,
            'status' => $this->status,
            'occupier_alias' => $this->occupier_alias,
            'position' => $this->getPositionArray(),
            'pinned' => $this->pinned,
            'solarsystem_id' => $this->solarsystem_id,
            'signatures_count' => $this->signatures_count,
            'wormhole_signatures_count' => $this->wormhole_signatures_count,
            'map_connections_count' => $this->map_connections_count,
            'signatures' => $this->whenLoaded('signatures', fn () => $this->signatures->toResourceCollection(MapSignatureResource::class)),
        ];
    }

    private function getPositionArray(): ?array
    {
        if ($this->position_x === null || $this->position_y === null) {
            return null;
        }

        return [
            'x' => $this->position_x,
            'y' => $this->position_y,
        ];
    }
}
