<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\MapConnection;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Throwable;

/**
 * @mixin MapConnection
 */
final class MapConnectionResource extends JsonResource
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
            'from_map_solarsystem_id' => $this->from_map_solarsystem_id,
            'to_map_solarsystem_id' => $this->to_map_solarsystem_id,
            'wormhole_id' => $this->wormhole_id,
            'mass_status' => $this->mass_status,
            'connected_at' => $this->connected_at,
            'ship_size' => $this->ship_size,
            'is_eol' => $this->is_eol,
            'created_at' => $this->created_at,
            'updated_at' => $this->updated_at,
            'signatures' => $this->signatures->toResourceCollection(SignatureResource::class),
        ];
    }
}
