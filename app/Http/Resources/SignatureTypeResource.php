<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\SignatureType;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin SignatureType
 */
final class SignatureTypeResource extends JsonResource
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
            'name' => $this->name,
            'signature' => $this->signature,
            'signature_category_id' => $this->signature_category_id,
            'target_class' => $this->target_class,
            'extra' => $this->extra,
            'spawn_areas' => $this->spawn_areas,
        ];
    }
}
