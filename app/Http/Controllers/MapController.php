<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Actions\Map\CreateMapAction;
use App\Actions\Map\UpdateMapAction;
use App\Enums\KillmailFilter;
use App\Enums\Permission;
use App\Enums\RoutePreference;
use App\Http\Requests\StoreMapRequest;
use App\Http\Requests\UpdateMapRequest;
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
use App\Models\MapUserSetting;
use App\Models\ShipHistory;
use App\Models\Solarsystem;
use App\Models\User;
use App\Scopes\CharacterHasMapAccess;
use App\Scopes\CharacterIsOnline;
use App\Scopes\UserAllowedMapTracking;
use App\Scopes\WithVisibleSolarsystems;
use App\Services\RouteOptions;
use App\Services\RouteService;
use Closure;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Relations\Relation;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Illuminate\Http\Resources\Json\ResourceCollection;
use Illuminate\Support\Facades\Gate;
use Illuminate\Support\Facades\Session;
use Illuminate\Support\Stringable;
use Inertia\Inertia;
use Inertia\Response;
use Throwable;

final class MapController extends Controller
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

        $settings = $this->getMapUserSettings($map->id);

        $solarsystems = fn (): ResourceCollection => $this->getSolarsystemsMatchingSearch($search);

        $selected_map_solarsystem_id = $request->integer('map_solarsystem_id');

        $selected_map_solarsystem = $this->getSelectedSolarsystem($map, $selected_map_solarsystem_id);

        $map_killmails = Inertia::defer(fn (): ResourceCollection => $this->getMapKills($map, $settings->killmail_filter));

        $can_view_characters = Gate::allows('viewCharacters', $map);

        $map_characters = fn (): ?ResourceCollection => $can_view_characters ? $this->getMapCharacters($map, $selected_map_solarsystem_id, $settings) : null;

        $ship_history = fn (): ?ResourceCollection => $can_view_characters ? $this->getShipHistory() : null;

        $map_route_solarsystems = Inertia::defer(fn (): array => $this->getMapRouteSolarsystems($map, $settings, $selected_map_solarsystem_id));

        $shortest_path = fn (): ?array => $this->getShortestPathFromRequest($request, $settings, $map);

        $closest_systems = fn (): array => [
            'results' => $this->getClosestSystemsFromRequest($request, $settings, $map),
            'from_system' => $this->getFromSystemForClosestSystems($request),
            'condition' => $request->string('condition', 'observatories')->toString(),
            'limit' => $request->integer('limit', 15),
        ];

        $tracking_origin = Inertia::optional(fn (): ?JsonResource => $this->getTrackingOrigin($request, $map));
        $tracking_target = Inertia::optional(fn (): ?JsonResource => $this->getTrackingTarget($request));

        return Inertia::render('maps/ShowMap', [
            'map' => $map->toResource(MapResource::class),
            'solarsystems' => $solarsystems,
            'search' => $search,
            'config' => config('map'),
            'selected_map_solarsystem' => fn (): ?JsonResource => $selected_map_solarsystem?->toResource(MapSolarsystemResource::class),
            'map_killmails' => $map_killmails,
            'map_characters' => $map_characters,
            'map_route_solarsystems' => $map_route_solarsystems,
            'ship_history' => $ship_history,
            'has_write_access' => Gate::allows('update', $map),
            'has_guest_access' => $map->getUserPermission($this->user) === Permission::Guest,
            'map_user_settings' => $settings->toResource(...),
            'ignored_systems' => $this->getIgnoredSystems(...),
            'shortest_path' => $shortest_path,
            'closest_systems' => $closest_systems,
            'tracking_origin' => $tracking_origin,
            'tracking_target' => $tracking_target,
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

    /**
     * @throws Throwable
     */
    private function getShipHistory(): ResourceCollection
    {
        return ShipHistory::query()
            ->where('ship_id', '=', CharacterStatus::query()
                ->where('character_id', '=', $this->user->active_character->id)
                ->select('ship_item_id'))
            ->latest()
            ->limit(10)
            ->get()
            ->toResourceCollection(ShipHistoryResource::class);
    }

    /**
     * @throws Throwable
     */
    private function getSolarsystemsMatchingSearch(Stringable $search): ResourceCollection
    {
        return Solarsystem::query()
            ->search($search)
            ->limit(10)
            ->get()
            ->toResourceCollection(SolarsystemResource::class);
    }

    private function getMapUserSettings(int $map_id): MapUserSetting
    {
        return $this->user->mapUserSettings()->firstOrCreate([
            'map_id' => $map_id,
        ]);
    }

    /**
     * @throws Throwable
     */
    private function getSelectedSolarsystem(Map $map, ?int $solarsystem_id): ?MapSolarsystem
    {
        if ($solarsystem_id === null || $solarsystem_id === 0) {
            return null;
        }

        $isGuest = $map->getUserPermission($this->user) === Permission::Guest;

        return $map->mapSolarsystems()
            ->with('signatures', 'wormholes')
            ->when(! $isGuest, fn ($query) => $query->with('audits', fn (Relation $query) => $query->latest()))
            ->findOrFail($solarsystem_id)
            ->hideNotes($isGuest);
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
    private function getMapCharacters(Map $map, ?int $map_solarsystem_id, MapUserSetting $map_user_setting): ResourceCollection
    {
        $map_solarsystem = $this->getSelectedSolarsystem($map, $map_solarsystem_id);

        return Character::query()
            ->hasTokenWithTrackingScopes()
            ->with('characterStatus')
            ->tap(new UserAllowedMapTracking($map))
            ->tap(new CharacterHasMapAccess($map, without_guests: true))
            ->tap(new CharacterIsOnline)
            ->get()
            ->map(function (Character $character) use ($map, $map_solarsystem, $map_user_setting): Character {
                if (! $map_solarsystem instanceof MapSolarsystem) {
                    return $character;
                }
                $character->route = $this->getFastestRoute(
                    $map_solarsystem->solarsystem_id ?? 0,
                    $character->characterStatus->solarsystem_id ?? 0,
                    $map_user_setting,
                    $map
                );

                return $character;
            })
            ->toResourceCollection(CharacterResource::class);
    }

    /**
     * @throws Throwable
     */
    private function getMapRouteSolarsystems(Map $map, MapUserSetting $mapUserSetting, ?int $map_solarsystem_id): array
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
            'route' => $this->getFastestRoute($current_solarsystem->id, $map_route_solarsystem->solarsystem_id, $mapUserSetting, $map),
        ])->all();
    }

    /**
     * Get the fastest route while respecting session-based ignored systems
     */
    private function getFastestRoute(int $start_solarsystem_id, int $destination_solarsystem_id, MapUserSetting $mapUserSetting, ?Map $map = null): array
    {
        $ignored_systems = Session::get('ignored_systems', []);

        $options = new RouteOptions(
            allowEol: $mapUserSetting->route_allow_eol,
            massStatus: $mapUserSetting->route_allow_mass_status,
            allowEveScout: $mapUserSetting->route_use_evescout,
            map: $map,
            ignoredSystems: $ignored_systems,
            routePreference: $mapUserSetting->route_preference ?? RoutePreference::Shorter,
            securityPenalty: $mapUserSetting->security_penalty ?? 50,
        );

        return $this->route_service->findRoute($start_solarsystem_id, $destination_solarsystem_id, $options);
    }

    private function getIgnoredSystems(): array
    {
        return Session::get('ignored_systems', []);
    }

    /**
     * @throws Throwable
     */
    private function getShortestPathFromRequest(Request $request, MapUserSetting $settings, Map $map): ?array
    {
        $from_solarsystem_id = $request->integer('from_solarsystem_id');
        $to_solarsystem_id = $request->integer('to_solarsystem_id');

        if (! $from_solarsystem_id || ! $to_solarsystem_id) {
            return null;
        }

        return $this->getShortestPath($from_solarsystem_id, $to_solarsystem_id, $settings, $map);
    }

    /**
     * @throws Throwable
     */
    private function getClosestSystemsFromRequest(Request $request, MapUserSetting $settings, Map $map): ?array
    {
        $fromSystemName = $request->string('from_system');
        $condition = $request->string('condition', 'observatories');
        $limit = $request->integer('limit', 15);
        $request->string('custom_condition');

        if ($fromSystemName->isEmpty()) {
            return null;
        }

        $fromSystem = Solarsystem::query()->where('name', $fromSystemName->toString())->first();

        if (! $fromSystem) {
            return null;
        }

        $conditionClosure = $this->getConditionClosure($condition->toString());

        if (! $conditionClosure instanceof Closure) {
            return null;
        }

        $options = new RouteOptions(
            allowEol: $settings->route_allow_eol,
            massStatus: $settings->route_allow_mass_status,
            allowEveScout: $settings->route_use_evescout,
            map: $map,
            ignoredSystems: Session::get('ignored_systems', []),
            routePreference: $settings->route_preference ?? RoutePreference::Shorter,
            securityPenalty: $settings->security_penalty ?? 50,
        );

        return $this->route_service->findClosestSystems($fromSystem->id, $options, $conditionClosure, $limit);
    }

    private function getConditionClosure(string $condition): ?Closure
    {
        return match ($condition) {
            'observatories' => fn ($query) => $query->where('has_jove_observatory', true),
            'highsec' => fn ($query) => $query->where('security', '>=', 0.5),
            'lowsec' => fn ($query) => $query->whereBetween('security', [0.1, 0.4]),
            'nullsec' => fn ($query) => $query->where('security', '<=', 0.0),
            default => null,
        };
    }

    private function getFromSystemForClosestSystems(Request $request): ?JsonResource
    {
        $fromSystemName = $request->string('from_system');

        if ($fromSystemName->isEmpty()) {
            return null;
        }

        return Solarsystem::query()
            ->with([
                'sovereignty' => ['alliance', 'corporation', 'faction'],
                'wormholeSystem.effect',
                'constellation',
                'region',
            ])
            ->where('name', $fromSystemName->toString())
            ->first()
            ?->toResource(SolarsystemResource::class);
    }

    /**
     * @throws Throwable
     */
    private function getShortestPath(int $from_solarsystem_id, int $to_solarsystem_id, MapUserSetting $mapUserSetting, Map $map): ?array
    {
        $route = $this->getFastestRoute($from_solarsystem_id, $to_solarsystem_id, $mapUserSetting, $map);

        if ($route === []) {
            return null;
        }

        $fromSolarsystem = Solarsystem::query()
            ->with([
                'sovereignty' => ['alliance', 'corporation', 'faction'],
                'wormholeSystem.effect',
                'constellation',
                'region',
            ])
            ->find($from_solarsystem_id);

        $toSolarsystem = Solarsystem::query()
            ->with([
                'sovereignty' => ['alliance', 'corporation', 'faction'],
                'wormholeSystem.effect',
                'constellation',
                'region',
            ])
            ->find($to_solarsystem_id);

        return [
            'from_solarsystem_id' => $from_solarsystem_id,
            'to_solarsystem_id' => $to_solarsystem_id,
            'from_solarsystem' => $fromSolarsystem?->toResource(SolarsystemResource::class),
            'to_solarsystem' => $toSolarsystem?->toResource(SolarsystemResource::class),
            'route' => $route,
            'jumps' => count($route),
        ];
    }

    /**
     * Get signatures for the tracking origin solarsystem
     *
     * @throws Throwable
     */
    private function getTrackingOrigin(Request $request, Map $map): ?JsonResource
    {
        $origin_map_solarsystem_id = $request->integer('origin_map_solarsystem_id');

        if (! $origin_map_solarsystem_id) {
            return null;
        }

        return $map->mapSolarsystems()
            ->where('map_solarsystems.id', $origin_map_solarsystem_id)
            ->with([
                'signatures' => fn ($query) => $query
                    ->whereHas('signatureCategory', fn (Builder $q) => $q->where('name', 'Wormhole'))
                    ->with(['signatureType', 'signatureCategory', 'wormhole', 'mapConnection']),
            ])
            ->first()->toResource(MapSolarsystemResource::class);
    }

    /**
     * Get the target solarsystem for tracking
     *
     * @throws Throwable
     */
    private function getTrackingTarget(Request $request): ?JsonResource
    {
        $target_solarsystem_id = $request->integer('target_solarsystem_id');

        if (! $target_solarsystem_id) {
            return null;
        }

        return Solarsystem::query()
            ->with([
                'sovereignty' => ['alliance', 'corporation', 'faction'],
                'wormholeSystem.effect',
                'constellation',
                'region',
            ])
            ->find($target_solarsystem_id)
            ?->toResource(SolarsystemResource::class);
    }
}
