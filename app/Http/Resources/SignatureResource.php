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
            'signature_type_id' => $this->signature_type_id,
            'signature_category_id' => $this->signature_category_id,
            'raw_type_name' => $this->unless($this->signature_type_id !== null, $this->raw_type_name),
            'signature_type' => $this->signatureType,
            'signature_category' => $this->signatureCategory,
            'map_connection_id' => $this->map_connection_id,
            'wormhole_id' => $this->wormhole_id,
            'mass_status' => $this->mass_status,
            'ship_size' => $this->ship_size,
            'lifetime' => $this->lifetime,
            'lifetime_updated_at' => $this->lifetime_updated_at,
            'created_at' => $this->created_at,
            'updated_at' => $this->updated_at,
            'wormhole' => $this->wormhole?->toResource(WormholeResource::class),
            'map_connection' => $this->whenLoaded('mapConnection', fn () => $this->mapConnection->toResource(MapConnectionResource::class)),
        ];
    }
}
