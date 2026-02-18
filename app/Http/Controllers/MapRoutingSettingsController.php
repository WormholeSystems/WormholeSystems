<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Features\MapSettingsFeature;
use App\Http\Resources\MapInfoResource;
use App\Models\Map;
use App\Models\MapUserSetting;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Support\Facades\Gate;
use Illuminate\Support\Facades\Session;
use Inertia\Inertia;
use Inertia\Response;
use Throwable;

final class MapRoutingSettingsController extends Controller
{
    public function __construct(
        #[CurrentUser] private readonly User $user,
    ) {}

    /**
     * Show the routing settings page
     *
     * @throws Throwable
     */
    public function show(Map $map): Response
    {
        Gate::authorize('view', $map); // Everyone with view access can see routing settings

        $settings = $this->getMapUserSettings($map->id);

        return Inertia::render('maps/settings/ShowRouting', [
            'map' => $map->toResource(MapInfoResource::class),
            'map_user_settings' => $settings->toResource(),
            'is_owner' => Gate::allows('delete', $map),
            'permission' => $map->getUserPermission($this->user)?->value,
        ]);
    }

    private function getMapUserSettings(int $mapId): MapUserSetting
    {
        $settings = $this->user->mapUserSettings()->firstOrCreate([
            'map_id' => $mapId,
        ]);

        Session::put(MapSettingsFeature::sessionKey($mapId), $settings->attributesToArray());

        return $settings;
    }
}
