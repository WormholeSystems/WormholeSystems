<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\Character;
use App\Models\EsiScope;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Throwable;

/**
 * @mixin Character
 */
final class CharacterResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     *
     * @throws Throwable
     */
    public function toArray(Request $request): array
    {
        return [
            'id' => $this->id,
            'name' => $this->name,
            'corporation' => $this->corporation?->toResource(CorporationResource::class),
            'alliance' => $this->alliance?->toResource(AllianceResource::class),
            'faction' => $this->faction?->toResource(FactionResource::class),
            'security_status' => $this->security_status,
            'status' => $this->whenLoaded('characterStatus', fn () => $this->characterStatus?->toResource(CharacterStatusResource::class)),
            'route' => $this->whenHas('route', fn () => $this->route),
            'esi_scopes' => $this->whenLoaded('esiScopes', fn () => $this->esiScopes->map(fn (EsiScope $scope) => $scope->name)),
        ];
    }
}
