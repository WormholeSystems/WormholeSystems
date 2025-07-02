<?php

namespace App\Http\Resources;

use App\Models\MapConnection;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin MapConnection
 */
class MapConnectionResource extends JsonResource
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
            'map_id' => $this->map_id,
            'from_map_solarsystem_id' => $this->from_map_solarsystem_id,
            'to_map_solarsystem_id' => $this->to_map_solarsystem_id,
            'wormhole_id' => $this->wormhole_id,
            'status' => $this->status,
            'connected_at' => $this->connected_at,
            'created_at' => $this->created_at,
            'updated_at' => $this->updated_at,
        ];
    }
}
