<?php

namespace App\Http\Controllers;

use App\Actions\Map\CreateMapAction;
use App\Http\Requests\StoreMapRequest;
use App\Http\Resources\CharacterResource;
use App\Http\Resources\KillmailResource;
use App\Http\Resources\MapResource;
use App\Http\Resources\MapSolarsystemResource;
use App\Http\Resources\SolarsystemResource;
use App\Models\Character;
use App\Models\Killmail;
use App\Models\Map;
use App\Models\MapAccess;
use App\Models\MapSolarsystem;
use App\Models\Solarsystem;
use App\Models\User;
use App\Scopes\WithVisibleSolarsystems;
use App\Services\RouteService;
use Illuminate\Container\Attributes\CurrentUser;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Illuminate\Http\Resources\Json\ResourceCollection;
use Illuminate\Support\Facades\App;
use Illuminate\Support\Facades\Gate;
use Inertia\Inertia;
use Inertia\Response;
use Throwable;

class MapController extends Controller
{
    public function __construct(
        #[CurrentUser] private readonly User $user,
    )
    {
    }

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

        $selected_map_solarsystem = fn(): ?JsonResource => $this->getSelectedSolarsystem($selected_map_solarsystem_id)?->toResource(MapSolarsystemResource::class);

        $map_killmails = Inertia::defer(
            fn(): ResourceCollection => $this->getMapKills($map)
        );

        $map_characters = fn(): ResourceCollection => $this->getMapCharacters($map);

        return Inertia::render('maps/ShowMap', [
            'map' => $map->toResource(MapResource::class),
            'solarsystems' => $solarsystems,
            'search' => $search,
            'config' => config('map'),
            'selected_map_solarsystem' => $selected_map_solarsystem,
            'map_killmails' => $map_killmails,
            'map_characters' => $map_characters,
            'jumps' => Inertia::defer(fn() => $this->getJumpsFromMapSolarsystem($map, $selected_map_solarsystem_id)),
        ]);
    }

    /**
     * @throws Throwable
     */
    public function index(): Response
    {
        return Inertia::render('maps/ShowAllMaps', [
            'maps' => Map::query()
                ->whereHas('mapAccessors', fn(Builder $builder) => $builder->whereIn('accessible_id', $this->user->getAccessibleIds()))
                ->withCount([
                    'mapSolarsystems' => fn(Builder $builder) => $builder->whereNotNull('position_x'),
                ])
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

    /**
     * @throws Throwable
     */
    private function getSelectedSolarsystem(?int $solarsystem_id): ?MapSolarsystem
    {
        if ($solarsystem_id === null || $solarsystem_id === 0) {
            return null;
        }

        return MapSolarsystem::query()
            ->with('signatures')
            ->findOrFail($solarsystem_id);
    }

    /**
     * @throws Throwable
     */
    private function getMapKills(Map $map): ResourceCollection
    {
        return Killmail::query()->with('shipType')
            ->whereIn('solarsystem_id', $map->mapSolarsystems->pluck('solarsystem_id'))
            ->orderBy('time', 'desc')
            ->limit(50)
            ->get()
            ->toResourceCollection(KillmailResource::class);
    }

    /**
     * @throws Throwable
     */
    private function getMapCharacters(Map $map): ResourceCollection
    {
        return Character::query()
            ->with('characterStatus')
            ->where(fn(Builder $query) => $query
                ->whereExists(MapAccess::query()
                    ->where('map_id', $map->id)
                    ->where(fn(Builder $query) => $query->
                    whereColumn('accessible_id', 'characters.id')
                        ->orWhereColumn('accessible_id', 'characters.corporation_id')
                        ->orWhereColumn('accessible_id', 'characters.alliance_id'
                        )
                    ))
            )
            ->whereRelation('characterStatus', 'is_online', true)
            ->get()
            ->toResourceCollection(CharacterResource::class);
    }

    /**
     * @throws Throwable
     */
    private function getJumpsFromMapSolarsystem(Map $map, ?int $map_solarsystem_id): array
    {
        $routeService = App::make(RouteService::class);
        if ($map_solarsystem_id === null || $map_solarsystem_id === 0) {
            return [];
        }

        $solarsystem_id = MapSolarsystem::query()
            ->where('id', $map_solarsystem_id)
            ->pluck('solarsystem_id');


        $targets = [
            'Jita',
            'Amarr',
            'Dodixie',
            'Rens',
            'Hek',
        ];

        $from = Solarsystem::query()->firstWhere('id', $solarsystem_id);
        $to = Solarsystem::query()->whereIn('name', $targets)
            ->get();

        return $to->map(fn(Solarsystem $solarsystem) => [
            'destination' => $solarsystem->toResource(SolarsystemResource::class),
            'route' => $routeService->find($from->id, $solarsystem->id, $map),
        ])->all();
    }
}
