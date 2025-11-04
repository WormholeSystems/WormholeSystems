<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Http\Resources\MapInfoResource;
use App\Models\Map;
use App\Models\MapUserSetting;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Support\Facades\Gate;
use Inertia\Inertia;
use Inertia\Response;

final class MapPreferencesController extends Controller
{
    public function __construct(
        #[CurrentUser] private readonly User $user,
    ) {}

    public function show(Map $map): Response
    {
        Gate::authorize('view', $map); // Everyone with view access can see preferences

        $settings = $this->getMapUserSettings($map->id);

        return Inertia::render('maps/settings/ShowPreferences', [
            'map' => $map->toResource(MapInfoResource::class),
            'map_user_settings' => $settings->toResource(),
            'is_owner' => Gate::allows('delete', $map),
            'has_write_access' => Gate::allows('update', $map),
        ]);
    }

    private function getMapUserSettings(int $mapId): MapUserSetting
    {
        return $this->user->mapUserSettings()->firstOrCreate([
            'map_id' => $mapId,
        ]);
    }
}
