<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\WormholeSystemThreat;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin WormholeSystemThreat
 *
 * @property string|null $occupier_alias Subquery alias selected by the threat search query.
 */
final class MapThreatSearchSystemResource extends JsonResource
{
    /**
     * @return array<string, mixed>
     */
    public function toArray(Request $request): array
    {
        return [
            'solarsystem_id' => $this->wormhole_system_id,
            'kills' => $this->kills,
            'occupier_alias' => $this->occupier_alias,
        ];
    }
}
