<?php

namespace App\Http\Controllers;

use App\Actions\Map\CreateMapAction;
use App\Enums\KillmailFilter;
use App\Http\Requests\StoreMapRequest;
use App\Http\Resources\CharacterResource;
use App\Http\Resources\KillmailResource;
use App\Http\Resources\MapResource;
use App\Http\Resources\MapSolarsystemResource;
use App\Http\Resources\ShipHistoryResource;
use App\Http\Resources\SolarsystemResource;
use App\Models\Character;
use App\Models\CharacterStatus;
use App\Models\Killmail;
use App\Models\Map;
use App\Models\MapRouteSolarsystem;
use App\Models\MapSolarsystem;
use App\Models\ShipHistory;
use App\Models\Solarsystem;
use App\Models\User;
use App\Scopes\CharacterHasMapAccess;
use App\Scopes\CharacterIsOnline;
use App\Scopes\UserAllowedMapTracking;
use App\Scopes\WithVisibleSolarsystems;
use App\Services\RouteOptions;
use App\Services\RouteService;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Illuminate\Http\Resources\Json\ResourceCollection;
use Illuminate\Support\Facades\Gate;
use Illuminate\Support\Facades\Session;
use Inertia\Inertia;
use Inertia\Response;
use Throwable;

class MapController extends Controller
{
    public function __construct(
        #[CurrentUser] private readonly User $user,
        private readonly RouteService $route_service,
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

        $search = $request->string('search');

        $solarsystems = Solarsystem::query()->whereLike('name', sprintf('%s%%', $search->value()))
            ->limit(10)->get()->toResourceCollection(SolarsystemResource::class);

        $selected_map_solarsystem_id = $request->integer('map_solarsystem_id');

        $selected_map_solarsystem = $this->getSelectedSolarsystem($map, $selected_map_solarsystem_id);

        $killmail_filter = KillmailFilter::tryFrom(Session::get('killmail_filter', 'all')) ?? KillmailFilter::All;

        $map_killmails = Inertia::defer(
            fn (): ResourceCollection => $this->getMapKills($map, $killmail_filter)
        );

        $map_characters = fn (): ResourceCollection => $this->getMapCharacters($map, $selected_map_solarsystem);

        $ship_history = ShipHistory::query()
            ->where('ship_id', '=', CharacterStatus::query()->where('character_id', '=', $this->user->active_character->id)
                ->select('ship_item_id'))
            ->latest()
            ->limit(10)
            ->get()
            ->toResourceCollection(ShipHistoryResource::class);

        return Inertia::render('maps/ShowMap', [
            'map' => $map->toResource(MapResource::class),
            'solarsystems' => $solarsystems,
            'search' => $search,
            'config' => config('map'),
            'selected_map_solarsystem' => fn (): ?JsonResource => $selected_map_solarsystem?->toResource(MapSolarsystemResource::class),
            'map_killmails' => $map_killmails,
            'map_characters' => $map_characters,
            'map_route_solarsystems' => Inertia::defer(fn (): array => $this->getMapRouteSolarsystems($map, $selected_map_solarsystem_id)),
            'ship_history' => $ship_history,
            'has_write_access' => Gate::allows('update', $map),
            'allow_eol' => Session::get('allow_eol', true),
            'allow_crit' => Session::get('allow_crit', true),
            'allow_eve_scout' => Session::get('allow_eve_scout', false),
            'ignored_systems' => Session::get('ignored_systems', []),
            'killmail_filter' => $killmail_filter,
        ]);
    }

    /**
     * @throws Throwable
     */
    public function index(): Response
    {
        return Inertia::render('maps/ShowAllMaps', [
            'maps' => Map::query()
                ->whereHas('mapAccessors', fn (Builder $builder) => $builder->whereIn('accessible_id', $this->user->getAccessibleIds()))
                ->withCount([
                    'mapSolarsystems' => fn (Builder $builder) => $builder->whereNotNull('position_x'),
                ])
                ->with('mapUserSetting')
                ->get()
                ->toResourceCollection(MapResource::class),
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

    public function destroy(Map $map): RedirectResponse
    {
        Gate::authorize('delete', $map);

        $map->delete();

        return to_route('home')->notify('Map deleted successfully.', 'You have successfully deleted the map.');
    }

    /**
     * @throws Throwable
     */
    private function getSelectedSolarsystem(Map $map, ?int $solarsystem_id): ?MapSolarsystem
    {
        if ($solarsystem_id === null || $solarsystem_id === 0) {
            return null;
        }

        return $map->mapSolarsystems()
            ->with('signatures')
            ->findOrFail($solarsystem_id);
    }

    /**
     * @throws Throwable
     */
    private function getMapKills(Map $map, KillmailFilter $filter): ResourceCollection
    {
        return Killmail::query()->with('shipType')
            ->whereIn('solarsystem_id', $map->mapSolarsystems->pluck('solarsystem_id'))
            ->when($filter === KillmailFilter::KSpace, fn (Builder $query) => $query->whereRelation('solarsystem', 'type', 'eve'))
            ->when($filter === KillmailFilter::JSpace, fn (Builder $query) => $query->whereRelation('solarsystem', 'type', 'wormhole'))
            ->orderByDesc('id')
            ->limit(50)
            ->get()
            ->toResourceCollection(KillmailResource::class);
    }

    /**
     * @throws Throwable
     */
    private function getMapCharacters(Map $map, ?MapSolarsystem $mapSolarsystem = null): ResourceCollection
    {
        return Character::query()
            ->with('characterStatus')
            ->tap(new UserAllowedMapTracking($map))
            ->tap(new CharacterHasMapAccess($map))
            ->tap(new CharacterIsOnline)
            ->get()
            ->map(function (Character $character) use ($mapSolarsystem, $map): Character {
                if (! $mapSolarsystem instanceof MapSolarsystem) {
                    return $character;
                }
                $character->route = $this->getFastestRoute(
                    $mapSolarsystem->solarsystem_id ?? 0,
                    $character->characterStatus?->solarsystem_id ?? 0,
                    $map
                );

                return $character;
            })
            ->toResourceCollection(CharacterResource::class);
    }

    /**
     * @throws Throwable
     */
    private function getMapRouteSolarsystems(Map $map, ?int $map_solarsystem_id): array
    {
        if ($map_solarsystem_id === null || $map_solarsystem_id === 0) {
            return [];
        }

        $solarsystem_id = MapSolarsystem::query()
            ->where('id', $map_solarsystem_id)
            ->pluck('solarsystem_id');

        $current_solarsystem = Solarsystem::query()
            ->firstWhere('id', $solarsystem_id);
        $map_route_solarsystems = $map->mapRouteSolarsystems;

        return $map_route_solarsystems->map(fn (MapRouteSolarsystem $map_route_solarsystem): array => [
            'id' => $map_route_solarsystem->id,
            'solarsystem' => $map_route_solarsystem->solarsystem->toResource(SolarsystemResource::class),
            'is_pinned' => $map_route_solarsystem->is_pinned,
            'route' => $this->getFastestRoute($current_solarsystem->id, $map_route_solarsystem->solarsystem_id, $map),
        ])->all();
    }

    /**
     * Get the fastest route while respecting session-based ignored systems
     */
    private function getFastestRoute(int $start_solarsystem_id, int $destination_solarsystem_id, ?Map $map = null): array
    {
        $allow_eol = Session::get('allow_eol', true);
        $allow_crit = Session::get('allow_crit', true);
        $allow_eve_scout = Session::get('allow_eve_scout', false);
        $ignored_systems = Session::get('ignored_systems', []);

        $options = new RouteOptions(
            allowEol: $allow_eol,
            allowCrit: $allow_crit,
            allowEveScout: $allow_eve_scout,
            map: $map,
            ignoredSystems: $ignored_systems
        );

        return $this->route_service->findRoute($start_solarsystem_id, $destination_solarsystem_id, $options);
    }
}
