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
            'from_map_solarsystem_id' => $this->from_map_solarsystem_id,
            'to_map_solarsystem_id' => $this->to_map_solarsystem_id,
            'mass_status' => $this->mass_status,
            'lifetime_status' => $this->lifetime,
            'lifetime_status_updated_at' => $this->lifetime_updated_at,
            'signatures' => $this->signatures->toResourceCollection(MapSignatureResource::class),
            'ship_size' => $this->ship_size,
            'created_at' => $this->created_at,
            'updated_at' => $this->updated_at,
        ];
    }
}
