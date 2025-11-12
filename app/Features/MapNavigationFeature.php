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
use Closure;
use Illuminate\Http\Resources\Json\JsonResource;
use Illuminate\Support\Facades\Session;
use Illuminate\Support\Stringable;
use Inertia\ProvidesInertiaProperties;
use Inertia\RenderContext;
use Throwable;

final readonly class MapNavigationFeature implements ProvidesInertiaProperties
{
    public function __construct(
        private Map $map,
        private ?MapSolarsystem $selected_map_solarsystem,
        private MapUserSetting $mapUserSetting,
        private RouteService $routeService,
        private Stringable $from_system,
        private Stringable $condition,
        private int $limit,
        private ?int $from_solarsystem_id,
        private ?int $to_solarsystem_id,
    ) {}

    public function toInertiaProperties(RenderContext $context): array
    {
        return [
            'map_navigation' => fn (): array => [
                'destinations' => $this->getDestinations(),
                'closest_systems' => [
                    'results' => $this->getClosestSystems(),
                    'from_system' => $this->getFromSystem(),
                    'condition' => $this->condition->toString(),
                    'limit' => $this->limit,
                ],
                'shortest_path' => $this->getShortestPath(),
            ],
        ];
    }

    /**
     * Get destination routes (formerly MapRouteFeature)
     *
     * @throws Throwable
     */
    private function getDestinations(): array
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

    /**
     * Get the closest systems (formerly MapClosestSystemsFeature)
     */
    private function getClosestSystems(): ?array
    {
        if ($this->from_system->isEmpty()) {
            return null;
        }

        $fromSystem = Solarsystem::query()->where('name', $this->from_system->toString())->first();

        if (! $fromSystem) {
            return null;
        }

        $conditionClosure = $this->getConditionClosure($this->condition->toString());

        if (! $conditionClosure instanceof Closure) {
            return null;
        }

        return $this->routeService->findClosestSystems(
            $fromSystem->id,
            $this->getRouteOptions(),
            $conditionClosure,
            $this->limit
        );
    }

    private function getFromSystem(): ?JsonResource
    {
        if ($this->from_system->isEmpty()) {
            return null;
        }

        return Solarsystem::query()
            ->with([
                'sovereignty' => ['alliance', 'corporation', 'faction'],
                'wormholeSystem.effect',
                'constellation',
                'region',
            ])
            ->where('name', $this->from_system->toString())
            ->first()
            ?->toResource(SolarsystemResource::class);
    }

    private function getConditionClosure(string $condition): ?Closure
    {
        return match ($condition) {
            'observatories' => fn ($query) => $query->where('has_jove_observatory', true),
            'npc_stations' => fn ($query) => $query->whereHas('stations'),
            'highsec' => fn ($query) => $query->where('security', '>=', 0.5),
            'lowsec' => fn ($query) => $query->whereBetween('security', [0.1, 0.4]),
            'nullsec' => fn ($query) => $query->where('security', '<=', 0.0),
            default => null,
        };
    }

    /**
     * Get the shortest path (formerly MapShortestPathFeature)
     *
     * @throws Throwable
     */
    private function getShortestPath(): ?array
    {
        if (! $this->from_solarsystem_id && ! $this->to_solarsystem_id) {
            return null;
        }

        $fromSolarsystem = null;
        $toSolarsystem = null;
        $route = [];

        if ($this->from_solarsystem_id) {
            $fromSolarsystem = Solarsystem::query()
                ->with([
                    'sovereignty' => ['alliance', 'corporation', 'faction'],
                    'wormholeSystem.effect',
                    'constellation',
                    'region',
                ])
                ->find($this->from_solarsystem_id);
        }

        if ($this->to_solarsystem_id) {
            $toSolarsystem = Solarsystem::query()
                ->with([
                    'sovereignty' => ['alliance', 'corporation', 'faction'],
                    'wormholeSystem.effect',
                    'constellation',
                    'region',
                ])
                ->find($this->to_solarsystem_id);
        }

        // Only calculate route if both systems are set and different
        if ($this->from_solarsystem_id && $this->to_solarsystem_id && $this->from_solarsystem_id !== $this->to_solarsystem_id) {
            $route = $this->routeService->findRoute(
                $this->from_solarsystem_id,
                $this->to_solarsystem_id,
                $this->getRouteOptions()
            );
        }

        return [
            'from_solarsystem_id' => $this->from_solarsystem_id,
            'to_solarsystem_id' => $this->to_solarsystem_id,
            'from_solarsystem' => $fromSolarsystem?->toResource(SolarsystemResource::class),
            'to_solarsystem' => $toSolarsystem?->toResource(SolarsystemResource::class),
            'route' => $route,
            'jumps' => count($route),
        ];
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
