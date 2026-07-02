<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\Signature;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * Slim signature shape embedded in map-wide connection payloads. Only the fields the
 * connection-level consumers use are included, keeping broadcast payloads small.
 *
 * @mixin Signature
 */
final class MapConnectionSignatureResource extends JsonResource
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
            'signature_id' => $this->signature_id,
            'map_solarsystem_id' => $this->map_solarsystem_id,
            'target_class' => $this->signatureType?->target_class,
            'raw_type_name' => $this->signatureType === null ? $this->raw_type_name : null,
            'mass_status' => $this->mass_status,
            'lifetime_status' => $this->lifetime,
            'lifetime_status_updated_at' => $this->lifetime_updated_at,
            'created_at' => $this->created_at,
            'updated_at' => $this->updated_at,
            'wormhole' => $this->wormhole?->toResource(WormholeResource::class),
        ];
    }
}
