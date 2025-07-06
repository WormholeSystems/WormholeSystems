<?php

namespace App\Http\Resources;

use App\Models\Faction;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin Faction
 */
class FactionResource extends JsonResource
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
        ];
    }
}
