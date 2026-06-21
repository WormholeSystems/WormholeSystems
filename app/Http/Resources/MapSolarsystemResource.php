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
            'status' => $this->details->status,
            'occupier_alias' => $this->details->occupier_alias,
            'position' => $this->getPositionArray(),
            'pinned' => $this->pinned,
            'solarsystem_id' => $this->solarsystem_id,
            'signatures_count' => $this->signatures_count,
            'uncategorized_signatures_count' => $this->uncategorized_signatures_count,
            'wormhole_signatures_count' => $this->wormhole_signatures_count,
            'map_connections_count' => $this->map_connections_count,
            'threat_level' => $this->whenLoaded('wormholeSystem', fn () => $this->wormholeSystem?->threat_level),
            'signatures' => $this->whenLoaded('signatures', fn () => $this->signatures->toResourceCollection(MapSignatureResource::class)),
        ];
    }

    /**
     * @return array{x: int, y: int}
     */
    private function getPositionArray(): array
    {
        return [
            'x' => $this->position_x,
            'y' => $this->position_y,
        ];
    }
}
