<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\Map\CreateMapAction;
use App\Actions\Map\UpdateMapAction;
use App\Features\EveScoutConnectionsFeature;
use App\Features\MapCharactersFeature;
use App\Features\MapClosestSystemsFeature;
use App\Features\MapKillmailsFeature;
use App\Features\MapPermissionsFeature;
use App\Features\MapRouteFeature;
use App\Features\MapSearchFeature;
use App\Features\MapSelectionFeature;
use App\Features\MapSettingsFeature;
use App\Features\MapShortestPathFeature;
use App\Features\MapTrackingFeature;
use App\Features\ShipHistoryFeature;
use App\Http\Requests\StoreMapRequest;
use App\Http\Requests\UpdateMapRequest;
use App\Http\Resources\MapResource;
use App\Models\Map;
use App\Models\User;
use App\Scopes\WithVisibleSolarsystems;
use App\Services\EveScoutService;
use App\Services\RouteService;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Gate;
use Inertia\Inertia;
use Inertia\Response;
use Throwable;

final class MapController extends Controller
{
    public function __construct(
        #[CurrentUser] private readonly User $user,
        private readonly RouteService $route_service,
        private readonly EveScoutService $eve_scout_service,
    ) {}

    /**
     * @throws Throwable
     */
    public function show(Request $request, Map $map): Response
    {
        Gate::authorize('view', $map);

        $map = Map::query()
            ->tap(new WithVisibleSolarsystems)
            ->findOrFail($map->id);

        // Initialize feature classes
        $settingsFeature = new MapSettingsFeature($this->user, $map->id);
        $settings = $settingsFeature->getSettings();

        $selectionFeature = new MapSelectionFeature(
            $map,
            $this->user,
            $request->integer('map_solarsystem_id')
        );

        $can_view_characters = Gate::allows('viewCharacters', $map);

        $permissionsFeature = new MapPermissionsFeature($map, $this->user);
        $searchFeature = new MapSearchFeature($request->string('search'));
        $killmailsFeature = new MapKillmailsFeature($map, $settings->killmail_filter);
        $charactersFeature = new MapCharactersFeature(
            $map,
            $can_view_characters,
            $selectionFeature->getSelectedMapSolarsystem(),
            $settings,
            $this->route_service
        );
        $shipHistoryFeature = new ShipHistoryFeature($this->user, $can_view_characters);
        $routeFeature = new MapRouteFeature(
            $map,
            $request->integer('map_solarsystem_id'),
            $settings,
            $this->route_service
        );
        $shortestPathFeature = new MapShortestPathFeature(
            $request->integer('from_solarsystem_id') ?: null,
            $request->integer('to_solarsystem_id') ?: null,
            $settings,
            $map,
            $this->route_service
        );
        $closestSystemsFeature = new MapClosestSystemsFeature(
            $request->string('from_system'),
            $request->string('condition', 'observatories'),
            $request->integer('limit', 15),
            $settings,
            $map,
            $this->route_service
        );
        $trackingFeature = new MapTrackingFeature(
            $map,
            $request->integer('origin_map_solarsystem_id') ?: null,
            $request->integer('target_solarsystem_id') ?: null
        );
        $eveScoutConnectionsFeature = new EveScoutConnectionsFeature(
            $this->eve_scout_service,
            $this->route_service,
            $map,
            $selectionFeature->getSelectedMapSolarsystem()
        );

        return Inertia::render('maps/ShowMap', [
            'map' => $map->toResource(MapResource::class),
            'config' => config('map'),
            $permissionsFeature,
            $settingsFeature,
            $searchFeature,
            $selectionFeature,
            $killmailsFeature,
            $charactersFeature,
            $shipHistoryFeature,
            $routeFeature,
            $shortestPathFeature,
            $closestSystemsFeature,
            $trackingFeature,
            $eveScoutConnectionsFeature,
        ]);
    }

    /**
     * @throws Throwable
     */
    public function index(Request $request): Response
    {
        $search = $request->string('search');

        return Inertia::render('maps/ShowAllMaps', [
            'maps' => Map::query()
                ->whereHas('mapAccessors', fn (Builder $builder) => $builder->whereIn('accessible_id', $this->user->getAccessibleIds()))
                ->when($search->isNotEmpty(), fn (Builder $query) => $query->whereLike('name', sprintf('%%%s%%', $search)))
                ->withCount([
                    'mapSolarsystems' => fn (Builder $builder) => $builder->whereNotNull('position_x'),
                ])
                ->with('mapUserSetting')
                ->get()
                ->toResourceCollection(MapResource::class),
            'search' => $search->toString(),
        ]);
    }

    public function create(): Response
    {
        Gate::authorize('create', Map::class);

        return Inertia::render('maps/CreateMap');
    }

    /**
     * @throws Throwable
     */
    public function store(StoreMapRequest $request, CreateMapAction $action): RedirectResponse
    {
        $map = $action->handle($this->user->active_character, $request->validated());

        return to_route('maps.show', $map)->notify('Map created successfully.', 'You have successfully created a new map.');
    }

    public function update(UpdateMapRequest $request, Map $map, UpdateMapAction $action): RedirectResponse
    {
        $action->handle($map, $request->validated());

        return back()->notify('Map updated successfully.', 'The map details have been updated.');
    }

    public function destroy(Map $map): RedirectResponse
    {
        Gate::authorize('delete', $map);

        $map->delete();

        return to_route('home')->notify('Map deleted successfully.', 'You have successfully deleted the map.');
    }
}
