<?php

namespace App\Http\Resources;

use App\Models\Signature;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin Signature
 */
class SignatureResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     * @return array<string, mixed>
     */
    public function toArray($request): array
    {
        return [
            'id' => $this->id,
            'signature_id' => $this->signature_id,
            'map_solarsystem_id' => $this->map_solarsystem_id,
            'name' => $this->name,
            'type' => $this->type,
            'category' => $this->category,
            'created_at' => $this->created_at,
            'updated_at' => $this->updated_at,
        ];
    }
}
