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
use App\Features\MapSkyhooksFeature;
use App\Features\MapTrackingFeature;
use App\Features\ShipHistoryFeature;
use App\Features\ThreatAnalysisFeature;
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
        #[CurrentUser] private readonly ?User $user,
        private readonly EveScoutService $eve_scout_service,
    ) {}

    /**
     * @throws Throwable
     */
    public function show(Request $request, Map $map): Response
    {
        Gate::authorize('view', $map);

        /** @var User|null $user */
        $user = $request->user();

        $map = Map::query()
            ->with([
                'mapSolarsystems' => fn (Relation $query): Relation => $query->withCount('signatures', 'wormholeSignatures', 'mapConnections', 'uncategorizedSignatures'),
                'mapRouteSolarsystems',
            ])
            ->tap(new WithVisibleSolarsystems)
            ->findOrFail($map->id);

        $selected_map_solarsystem = $this->getSelectedMapSolarsystem(
            $request->input('solarsystem_id'),
            $map,
        );

        $settingsFeature = new MapSettingsFeature($user, $map->id);
        $settings = $settingsFeature->getSettings();
        $hiddenCards = $settings->hidden_cards ?? [];
        $canViewCharacters = Gate::allows('viewCharacters', $map);

        return Inertia::render('maps/ShowMap', [
            'map' => $map->toResource(MapResource::class),
            'config' => config('map'),
        ])
            ->with(new MapPermissionsFeature($map, $user))
            ->with($settingsFeature)
            ->with(new MapSelectionFeature($map, $user, $selected_map_solarsystem, $hiddenCards))
            ->with(new MapTrackingFeature($map, $request->integer('origin_solarsystem_id') ?: null, $request->integer('target_solarsystem_id') ?: null))
            ->with(new MapCharactersFeature($map, $canViewCharacters))
            ->with(new EveScoutConnectionsFeature($this->eve_scout_service))
            ->with(new MapKillmailsFeature($map, $settings->killmail_filter ?? KillmailFilter::All, $hiddenCards))
            ->with(new ShipHistoryFeature($user, $canViewCharacters, $hiddenCards))
            ->with(new MapNavigationFeature($map, $hiddenCards))
            ->with(new ThreatAnalysisFeature($selected_map_solarsystem, $hiddenCards))
            ->with(new MapSkyhooksFeature($hiddenCards));
    }

    public function showByToken(string $token): RedirectResponse
    {
        $map = Map::query()->where('share_token', $token)->firstOrFail();

        return redirect()->route('maps.show', ['map' => $map, 'share_token' => $token]);
    }

    /**
     * @throws Throwable
     */
    public function index(Request $request): Response
    {
        $search = $request->string('search');
        $accessibleIds = $this->user->getAccessibleIds();

        return Inertia::render('maps/ShowAllMaps', [
            'maps' => Map::query()
                ->whereHas('mapAccessors', fn (Builder $builder) => $builder->notExpired()->whereIn('accessible_id', $accessibleIds))
                ->when($search->isNotEmpty(), fn (Builder $query) => $query->whereLike('name', sprintf('%%%s%%', $search)))
                ->withCount([
                    'mapSolarsystems' => fn (Builder $builder) => $builder->whereNotNull('position_x'),
                    'mapConnections',
                ])
                ->with([
                    'mapUserSetting',
                    'mapAccessors' => fn ($builder) => $builder->notExpired()->whereIn('accessible_id', $accessibleIds),
                ])
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
     * Resolve the selected system from its solarsystem id. Keying on the solarsystem (rather
     * than the placement) keeps the selection stable across a system being removed and
     * re-added, and simply clears when the system is not currently on the map.
     */
    private function getSelectedMapSolarsystem(null|string|int $selected_solarsystem_id, Map $map): ?MapSolarsystem
    {
        if ($selected_solarsystem_id === null) {
            return null;
        }

        return $map->mapSolarsystems()->where('solarsystem_id', (int) $selected_solarsystem_id)->first();
    }
}
