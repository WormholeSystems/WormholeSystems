<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\Signature;
use Illuminate\Http\Resources\Json\JsonResource;
use Throwable;

/**
 * @mixin Signature
 */
final class SignatureResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     * @return array<string, mixed>
     *
     * @throws Throwable
     */
    public function toArray($request): array
    {
        return [
            'id' => $this->id,
            'signature_id' => $this->signature_id,
            'map_solarsystem_id' => $this->map_solarsystem_id,
            'type' => $this->type,
            'category' => $this->category,
            'map_connection_id' => $this->map_connection_id,
            'wormhole_id' => $this->wormhole_id,
            'mass_status' => $this->mass_status,
            'ship_size' => $this->ship_size,
            'is_eol' => $this->is_eol,
            'created_at' => $this->created_at,
            'updated_at' => $this->updated_at,
            'wormhole' => $this->wormhole?->toResource(WormholeResource::class),
        ];
    }
}
