<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\Sovereignty;
use Illuminate\Http\Resources\Json\JsonResource;
use Throwable;

/**
 * @mixin Sovereignty
 */
final class SovereigntyResource extends JsonResource
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
            'alliance' => $this->alliance?->toResource(AllianceResource::class),
            'corporation' => $this->corporation?->toResource(CorporationResource::class),
            'faction' => $this->faction?->toResource(FactionResource::class),
        ];
    }
}
