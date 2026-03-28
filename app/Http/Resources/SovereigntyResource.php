<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\Sovereignty;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin Sovereignty
 */
final class SovereigntyResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     * @return array<string, mixed>
     */
    public function toArray(Request $request): array
    {
        return [
            'id' => $this->solarsystem_id,
            'alliance' => $this->whenLoaded('alliance', fn () => AllianceResource::make($this->alliance)),
            'corporation' => $this->whenLoaded('corporation', fn () => CorporationResource::make($this->corporation)),
            'faction' => $this->whenLoaded('faction', fn () => FactionResource::make($this->faction)),
        ];
    }
}
