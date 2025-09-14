<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\MapSolarsystem;
use App\Models\WormholeStatic;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Throwable;

use function in_array;

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
            'name' => $this->solarsystem->name,
            'solarsystem_id' => $this->solarsystem_id,
            'alias' => $this->alias,
            'occupier_alias' => $this->occupier_alias,
            'position' => $this->position_x !== null ? [
                'x' => $this->position_x,
                'y' => $this->position_y,
            ] : null,
            'status' => $this->status,
            'pinned' => $this->pinned,
            'class' => $this->wormholeSystem?->class,
            'effect' => $this->wormholeSystem?->effect?->name,
            'effects' => $this->wormholeSystem?->effect?->getEffectArray($this->wormholeSystem->class),
            'map_connections' => $this->connections?->toResourceCollection(MapConnectionResource::class),
            'solarsystem' => $this->solarsystem->toResource(SolarsystemResource::class),
            'statics' => $this->wormholeSystem?->wormholeStatics?->map(fn (WormholeStatic $static) => $static->wormhole->toResource(WormholeResource::class)),
            'signatures' => $this->whenLoaded('signatures', fn () => $this->signatures->toResourceCollection(SignatureResource::class)),
            'signatures_count' => $this->whenCounted('signatures', fn () => $this->signatures_count),
            'audits' => $this->whenLoaded('audits', fn () => $this->audits->toResourceCollection(AuditResource::class)),
            'wormholes' => $this->whenLoaded('wormholes', fn () => $this->wormholes->toResourceCollection(WormholeResource::class)),
            'notes' => $this->getNotes(),
        ];
    }

    /**
     * Get the notes for the map.
     */
    private function getNotes(): ?string
    {
        $hidden = $this->getHidden();

        if (in_array('notes', $hidden, true)) {
            return null;
        }

        return $this->notes;
    }
}
