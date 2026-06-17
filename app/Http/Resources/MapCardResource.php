<?php

declare(strict_types=1);

namespace App\Http\Resources;

use App\Models\Map;
use App\Models\MapAccess;
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
            'is_public' => $this->is_public,
            'role' => $this->resolveRole(),
            'map_solarsystems_count' => $this->map_solarsystems_count,
            'map_connections_count' => $this->map_connections_count,
            'map_user_setting' => $this->handleUserSetting(),
        ];
    }

    /**
     * The current user's highest role on the map, derived from the eager-loaded
     * (user-scoped) access entries. Owners are reported as "owner".
     */
    private function resolveRole(): ?string
    {
        $accessors = $this->mapAccessors;

        if ($accessors->isEmpty()) {
            return null;
        }

        if ($accessors->contains(fn (MapAccess $access): bool => (bool) $access->is_owner)) {
            return 'owner';
        }

        return $accessors
            ->sortByDesc(fn (MapAccess $access): int => $access->permission->level())
            ->first()
            ?->permission
            ?->value;
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
