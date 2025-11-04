<?php

declare(strict_types=1);

namespace App\Features;

use App\Enums\RoutePreference;
use App\Http\Resources\SolarsystemResource;
use App\Models\Map;
use App\Models\MapRouteSolarsystem;
use App\Models\MapSolarsystem;
use App\Models\MapUserSetting;
use App\Models\Solarsystem;
use App\Services\RouteOptions;
use App\Services\RouteService;
use Illuminate\Support\Facades\Session;
use Inertia\ProvidesInertiaProperties;
use Inertia\RenderContext;
use Throwable;

final readonly class MapRouteFeature implements ProvidesInertiaProperties
{
    public function __construct(
        private Map $map,
        private ?MapSolarsystem $selected_map_solarsystem,
        private MapUserSetting $mapUserSetting,
        private RouteService $routeService,
    ) {}

    public function toInertiaProperties(RenderContext $context): array
    {
        return [
            'map_route_solarsystems' => $this->getMapRouteSolarsystems(...),
        ];
    }

    /**
     * @throws Throwable
     */
    private function getMapRouteSolarsystems(): array
    {
        if (! $this->selected_map_solarsystem instanceof MapSolarsystem) {
            return [];
        }

        $solarsystem_id = Solarsystem::query()
            ->whereIn('id', fn ($query) => $query
                ->from('map_solarsystems')
                ->select('solarsystem_id')
                ->where('id', $this->selected_map_solarsystem->id)
            )
            ->value('id');

        if (! $solarsystem_id) {
            return [];
        }

        $current_solarsystem = Solarsystem::query()->findOrFail($solarsystem_id);
        $map_route_solarsystems = $this->map->mapRouteSolarsystems;

        // Build route requests for all destinations
        $routeRequests = [];
        foreach ($map_route_solarsystems as $map_route_solarsystem) {
            $routeRequests[$map_route_solarsystem->id] = [
                'from' => $current_solarsystem->id,
                'to' => $map_route_solarsystem->solarsystem_id,
            ];
        }

        // Calculate all routes in one batched operation
        $routes = [];
        if ($routeRequests !== []) {
            $routes = $this->routeService->findMultipleRoutes($routeRequests, $this->getRouteOptions());
        }

        return $map_route_solarsystems->map(fn (MapRouteSolarsystem $map_route_solarsystem): array => [
            'id' => $map_route_solarsystem->id,
            'solarsystem' => $map_route_solarsystem->solarsystem->toResource(SolarsystemResource::class),
            'is_pinned' => $map_route_solarsystem->is_pinned,
            'route' => $routes[$map_route_solarsystem->id] ?? [],
        ])->all();
    }

    private function getRouteOptions(): RouteOptions
    {
        $ignored_systems = Session::get('ignored_systems', []);

        return new RouteOptions(
            allowEol: $this->mapUserSetting->route_allow_eol,
            massStatus: $this->mapUserSetting->route_allow_mass_status,
            allowEveScout: $this->mapUserSetting->route_use_evescout,
            map: $this->map,
            ignoredSystems: $ignored_systems,
            routePreference: $this->mapUserSetting->route_preference ?? RoutePreference::Shorter,
            securityPenalty: $this->mapUserSetting->security_penalty ?? 50,
        );
    }
}
