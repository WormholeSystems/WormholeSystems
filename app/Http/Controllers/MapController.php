<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\Map\CreateMapAction;
use App\Actions\Map\UpdateMapAction;
use App\Enums\KillmailFilter;
use App\Features\EveScoutConnectionsFeature;
use App\Features\MapCharactersFeature;
use App\Features\MapKillmailsFeature;
use App\Features\MapNavigationFeature;
use App\Features\MapPermissionsFeature;
use App\Features\MapSelectionFeature;
use App\Features\MapSettingsFeature;
use App\Features\MapTrackingFeature;
use App\Features\ShipHistoryFeature;
use App\Http\Requests\StoreMapRequest;
use App\Http\Requests\UpdateMapRequest;
use App\Http\Resources\MapCardResource;
use App\Http\Resources\MapResource;
use App\Models\Map;
use App\Models\MapSolarsystem;
use App\Models\User;
use App\Scopes\WithVisibleSolarsystems;
use App\Services\EveScoutService;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Relations\Relation;
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
        private readonly EveScoutService $eve_scout_service,
    ) {}

    /**
     * @throws Throwable
     */
    public function show(Request $request, Map $map): Response
    {
        Gate::authorize('view', $map);
        $map = Map::query()
            ->with([
                'mapSolarsystems' => fn (Relation $query): Relation => $query->withCount('signatures', 'wormholeSignatures', 'mapConnections'),
                'mapRouteSolarsystems',
            ])
            ->tap(new WithVisibleSolarsystems)
            ->findOrFail($map->id);

        $selected_map_solarsystem = $this->getSelectedMapSolarsystem(
            $request->input('map_solarsystem_id'),
            $map,
        );

        $settingsFeature = new MapSettingsFeature($this->user, $map->id);
        $settings = $settingsFeature->getSettings();
        $hiddenCards = $settings->hidden_cards ?? [];
        $canViewCharacters = Gate::allows('viewCharacters', $map);

        return Inertia::render('maps/ShowMap', [
            'map' => $map->toResource(MapResource::class),
            'config' => config('map'),
        ])
            ->with(new MapPermissionsFeature($map, $this->user))
            ->with($settingsFeature)
            ->with(new MapSelectionFeature($map, $this->user, $selected_map_solarsystem, $hiddenCards))
            ->with(new MapTrackingFeature($map, $request->integer('origin_map_solarsystem_id') ?: null, $request->integer('target_solarsystem_id') ?: null))
            ->with(new MapCharactersFeature($map, $canViewCharacters))
            ->with(new EveScoutConnectionsFeature($this->eve_scout_service))
            ->with(new MapKillmailsFeature($map, $settings->killmail_filter ?? KillmailFilter::All, $hiddenCards))
            ->with(new ShipHistoryFeature($this->user, $canViewCharacters, $hiddenCards))
            ->with(new MapNavigationFeature($map, $hiddenCards));
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
                ->toResourceCollection(MapCardResource::class),
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

    /**
     * Get the selected map solarsystem based on the provided ID.
     */
    private function getSelectedMapSolarsystem(null|string|int $selected_map_solarsystem_id, Map $map): ?MapSolarsystem
    {
        if ($selected_map_solarsystem_id === null) {
            return null;
        }

        return $map->mapSolarsystems()->findOrFail((int) $selected_map_solarsystem_id);
    }
}
