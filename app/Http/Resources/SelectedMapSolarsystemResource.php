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
            'status' => $this->details->status,
            'occupier_alias' => $this->details->occupier_alias,
            'notes' => in_array('notes', $this->details->getHidden(), true) ? null : $this->details->notes,
            'position' => $this->getPositionArray(),
            'is_pinned' => $this->pinned,
            'map_connections' => $this->mapConnections->toResourceCollection(MapConnectionResource::class),
            'signatures' => $this->signatures->toResourceCollection(SignatureResource::class),
            'audits' => $this->details->audits->toResourceCollection(AuditResource::class),
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
