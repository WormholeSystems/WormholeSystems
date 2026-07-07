<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\WormholeSystemThreat;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Illuminate\Support\Collection;

/**
 * Wraps one entity's group of WormholeSystemThreat rows.
 *
 * @property Collection<int, WormholeSystemThreat> $resource
 */
final class MapThreatSearchResource extends JsonResource
{
    private const int SYSTEM_LIMIT = 10;

    /**
     * @return array<string, mixed>
     */
    public function toArray(Request $request): array
    {
        /** @var WormholeSystemThreat $first */
        $first = $this->resource->first();

        return [
            'id' => $first->entity_id,
            'name' => $first->name,
            'type' => $first->entity_type,
            'total_kills' => (int) $this->resource->sum('kills'),
            'systems_count' => $this->resource->count(),
            'systems' => $this->resource
                ->sortByDesc('kills')
                ->take(self::SYSTEM_LIMIT)
                ->values()
                ->toResourceCollection(MapThreatSearchSystemResource::class),
        ];
    }
}
