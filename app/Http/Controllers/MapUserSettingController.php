<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Events\Characters\CharacterStatusUpdatedEvent;
use App\Features\MapSettingsFeature;
use App\Http\Requests\UpdateMapUserSettingRequest;
use App\Models\Map;
use App\Models\MapUserSetting;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\RedirectResponse;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Session;
use Throwable;

final class MapUserSettingController extends Controller
{
    /**
     * @throws Throwable
     */
    public function update(UpdateMapUserSettingRequest $request, Map $map, #[CurrentUser] ?User $user): RedirectResponse
    {
        $validated = $request->validated();
        $sessionKey = MapSettingsFeature::sessionKey($map->id);

        if ($user instanceof User) {
            // Authenticated: update DB, then refresh session from full DB state
            $settings = DB::transaction(function () use ($user, $map, $validated): MapUserSetting {
                $settings = $user->mapUserSettings()->firstOrCreate(['map_id' => $map->id]);
                $settings->update($validated);

                broadcast(new CharacterStatusUpdatedEvent($map->id))->toOthers();

                return $settings->fresh();
            });

            Session::put($sessionKey, $settings->attributesToArray());
        } else {
            // Guest: merge into session only
            $current = Session::get($sessionKey, array_merge(MapSettingsFeature::DEFAULTS, ['map_id' => $map->id]));
            Session::put($sessionKey, array_merge($current, $validated));
        }

        return back();
    }
}
