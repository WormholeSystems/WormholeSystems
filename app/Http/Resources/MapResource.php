<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\Map;
use App\Models\MapUserSetting;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Throwable;

/**
 * @mixin Map
 */
final class MapResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     * @return array<string, mixed>
     *
     * @throws Throwable
     */
    public function toArray(Request $request): array
    {
        return [
            'id' => $this->id,
            'name' => $this->name,
            'slug' => $this->slug,
            'map_solarsystems' => $this->whenLoaded('mapSolarsystems', fn () => $this->mapSolarsystems->toResourceCollection(MapSolarsystemResource::class
            )),
            'map_connections' => $this->whenLoaded('mapConnections', fn () => $this->mapConnections->toResourceCollection(MapConnectionResource::class)),
            'map_solarsystems_count' => $this->whenCounted('mapSolarsystems', fn () => $this->map_solarsystems_count),
            'map_user_setting' => $this->handleUserSetting(),
            'owner' => $this->mapOwner->accessible->toResource(CharacterResource::class),
        ];
    }

    /**
     * Handle the user setting for the map.
     *
     * @throws Throwable
     */
    protected function handleUserSetting(): ?JsonResource
    {
        if ($this->mapUserSetting) {
            return $this->mapUserSetting->toResource(MapUserSettingResource::class);
        }

        $user_setting = MapUserSetting::query()->updateOrCreate([
            'user_id' => auth()->id(),
            'map_id' => $this->id,
        ]);

        return $user_setting->toResource(MapUserSettingResource::class);
    }
}
