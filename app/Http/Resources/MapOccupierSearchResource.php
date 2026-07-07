<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\MapSolarsystemDetails;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * @mixin MapSolarsystemDetails
 */
final class MapOccupierSearchResource extends JsonResource
{
    /**
     * @return array<string, mixed>
     */
    public function toArray(Request $request): array
    {
        return [
            'solarsystem_id' => $this->solarsystem_id,
            'occupier_alias' => $this->occupier_alias,
        ];
    }
}
