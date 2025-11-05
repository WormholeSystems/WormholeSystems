<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\Signature;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin Signature
 */
final class MapSignatureResource extends JsonResource
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
            'target_class' => $this->signatureType?->target_class,
            'extra' => $this->signatureType?->extra,
            'wormhole' => $this->wormhole?->toResource(WormholeResource::class),
            'category' => $this->getSignatureCategory(),
            'type' => $this->signatureType?->toResource(SignatureTypeResource::class),
            'raw_type_name' => $this->getRawTypeName(),
            'created_at' => $this->created_at,
            'updated_at' => $this->updated_at,
            'lifetime_status' => $this->lifetime,
            'lifetime_status_updated_at' => $this->lifetime_updated_at,
            'mass_status' => $this->mass_status,
            'map_connection_id' => $this->map_connection_id,
        ];
    }

    private function getSignatureCategory(): ?array
    {
        if ($this->signatureType === null) {
            return null;
        }

        return [
            'id' => $this->signatureType->category->id,
            'name' => $this->signatureType->category->name,
        ];
    }

    private function getSignatureType(): ?array
    {
        if ($this->signatureType === null) {
            return null;
        }

        return [
            'id' => $this->signatureType->id,
            'name' => $this->signatureType->name,
        ];
    }

    private function getRawTypeName(): ?string
    {
        if ($this->signatureType !== null) {
            return null;
        }

        return $this->raw_type_name;
    }
}
