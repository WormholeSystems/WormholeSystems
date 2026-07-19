<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\Map;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/**
 * The map's own attributes without its systems, connections or owner, so metadata
 * changes can be broadcast without the full topology.
 *
 * @mixin Map
 */
final class MapMetadataResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     * @return array<string, mixed>
     */
    public function toArray(Request $request): array
    {
        return [
            'id' => $this->id,
            'name' => $this->name,
            'slug' => $this->slug,
            'home_solarsystem_id' => $this->home_solarsystem_id,
            'rally_solarsystem_id' => $this->rally_solarsystem_id,
            'layout' => $this->layout,
            'allow_layout_override' => $this->allow_layout_override,
            'constant_width_enabled' => $this->constant_width_enabled,
            'bookmark_format_wormhole' => $this->bookmark_format_wormhole,
            'bookmark_format_kspace' => $this->bookmark_format_kspace,
            'bookmark_alias_scheme' => $this->bookmark_alias_scheme,
        ];
    }
}
