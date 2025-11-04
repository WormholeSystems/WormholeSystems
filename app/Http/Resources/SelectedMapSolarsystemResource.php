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
final class SelectedMapSolarsystemResource extends JsonResource
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
            'solarsystem_id' => $this->solarsystem_id,
            'alias' => $this->alias,
            'status' => $this->status,
            'occupier_alias' => $this->occupier_alias,
            'notes' => $this->notes,
            'position' => $this->getPositionArray(),
            'is_pinned' => $this->pinned,
            'solarsystem' => $this->solarsystem->toResource(SolarsystemResource::class),
            'map_connections' => $this->mapConnections->toResourceCollection(MapConnectionResource::class),
            'signatures' => $this->signatures->toResourceCollection(SignatureResource::class),
            'audits' => $this->audits->toResourceCollection(AuditResource::class),
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
