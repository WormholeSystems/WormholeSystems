<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\Map;
use App\Models\MapUserSetting;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Throwable;

use function auth;

/**
 * @mixin Map
 */
final class MapCardResource extends JsonResource
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
            'map_user_setting' => $this->handleUserSetting(),
            'owner' => $this->mapOwner->accessible->toResource(CharacterResource::class),
            'map_solarsystems_count' => $this->map_solarsystems_count,
        ];
    }

    /**
     * Handle the user setting for the map.
     *
     * @throws Throwable
     */
    private function handleUserSetting(): JsonResource
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
