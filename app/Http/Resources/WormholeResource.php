<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\Wormhole;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin Wormhole
 */
final class WormholeResource extends JsonResource
{
    public function toArray($request): array
    {
        return [
            'id' => $this->id,
            'name' => $this->name,
            'total_mass' => $this->total_mass,
            'maximum_jump_mass' => $this->maximum_jump_mass,
            'ship_size' => $this->ship_size,
            'maximum_lifetime' => $this->maximum_lifetime,
            'leads_to' => $this->leads_to,
            'type_id' => $this->type_id,
            'created_at' => $this->created_at,
            'updated_at' => $this->updated_at,
        ];
    }
}
