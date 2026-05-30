<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\MapIgnoredSolarsystems\CreateMapIgnoredSolarsystemAction;
use App\Actions\MapIgnoredSolarsystems\DeleteMapIgnoredSolarsystemAction;
use App\Events\MapIgnoredSolarsystemsUpdatedEvent;
use App\Features\MapSettingsFeature;
use App\Http\Requests\StoreMapIgnoredSolarsystemRequest;
use App\Http\Resources\MapInfoResource;
use App\Models\Map;
use App\Models\MapIgnoredSolarsystem;
use App\Models\MapUserSetting;
use App\Models\User;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Http\RedirectResponse;
use Illuminate\Support\Facades\Gate;
use Illuminate\Support\Facades\Session;
use Inertia\Inertia;
use Inertia\Response;
use Throwable;

final class MapIgnoredSolarsystemController extends Controller
{
    public function __construct(
        #[CurrentUser] private readonly User $user,
    ) {}

    /**
     * Show the mapping settings page (tracking + ignored systems).
     */
    public function show(Map $map): Response
    {
        Gate::authorize('view', $map); // Everyone with view access can see mapping settings

        $settings = $this->getMapUserSettings($map->id);

        return Inertia::render('maps/settings/ShowMapping', [
            'map' => $map->toResource(MapInfoResource::class),
            'map_user_settings' => $settings->toResource(),
            'map_ignored_systems' => $map->mapIgnoredSolarsystems()->pluck('solarsystem_id')->all(),
            'permission' => $map->getUserPermission($this->user)?->value,
        ]);
    }

    /**
     * @throws Throwable
     */
    public function store(StoreMapIgnoredSolarsystemRequest $request, CreateMapIgnoredSolarsystemAction $action): RedirectResponse
    {
        $action->handle($request->validated());

        return back()->notify('System ignored', message: 'The system will no longer be auto-mapped.');
    }

    /**
     * @throws Throwable
     */
    public function destroy(Map $map, int $solarsystem_id, DeleteMapIgnoredSolarsystemAction $action): RedirectResponse
    {
        Gate::authorize('update', $map);

        $mapIgnoredSolarsystem = $map->mapIgnoredSolarsystems()
            ->where('solarsystem_id', $solarsystem_id)
            ->first();

        if ($mapIgnoredSolarsystem instanceof MapIgnoredSolarsystem) {
            $action->handle($mapIgnoredSolarsystem);
        }

        return back()->notify('System unignored', message: 'The system can be auto-mapped again.');
    }

    public function destroyAll(Map $map): RedirectResponse
    {
        Gate::authorize('update', $map);

        $map->mapIgnoredSolarsystems()->delete();

        broadcast(new MapIgnoredSolarsystemsUpdatedEvent($map->id))->toOthers();

        return back()->notify('Ignore list cleared', message: 'All systems have been removed from the ignore list.');
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
