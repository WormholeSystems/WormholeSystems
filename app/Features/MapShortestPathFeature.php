<?php

declare(strict_types=1);

namespace App\Features;

use App\Enums\RoutePreference;
use App\Http\Resources\SolarsystemResource;
use App\Models\Map;
use App\Models\MapUserSetting;
use App\Models\Solarsystem;
use App\Services\RouteOptions;
use App\Services\RouteService;
use Illuminate\Support\Facades\Session;
use Inertia\ProvidesInertiaProperties;
use Inertia\RenderContext;
use Throwable;

final readonly class MapShortestPathFeature implements ProvidesInertiaProperties
{
    public function __construct(
        private ?int $from_solarsystem_id,
        private ?int $to_solarsystem_id,
        private MapUserSetting $mapUserSetting,
        private Map $map,
        private RouteService $routeService,
    ) {}

    public function toInertiaProperties(RenderContext $context): array
    {
        return [
            'shortest_path' => $this->getShortestPath(...),
        ];
    }

    /**
     * @throws Throwable
     */
    private function getShortestPath(): ?array
    {
        if (! $this->from_solarsystem_id || ! $this->to_solarsystem_id) {
            return null;
        }

        $route = $this->getFastestRoute(
            $this->from_solarsystem_id,
            $this->to_solarsystem_id
        );

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
            ->find($this->from_solarsystem_id);

        $toSolarsystem = Solarsystem::query()
            ->with([
                'sovereignty' => ['alliance', 'corporation', 'faction'],
                'wormholeSystem.effect',
                'constellation',
                'region',
            ])
            ->find($this->to_solarsystem_id);

        return [
            'from_solarsystem_id' => $this->from_solarsystem_id,
            'to_solarsystem_id' => $this->to_solarsystem_id,
            'from_solarsystem' => $fromSolarsystem?->toResource(SolarsystemResource::class),
            'to_solarsystem' => $toSolarsystem?->toResource(SolarsystemResource::class),
            'route' => $route,
            'jumps' => count($route),
        ];
    }

    private function getFastestRoute(int $start_solarsystem_id, int $destination_solarsystem_id): array
    {
        $ignored_systems = Session::get('ignored_systems', []);

        $options = new RouteOptions(
            allowEol: $this->mapUserSetting->route_allow_eol,
            massStatus: $this->mapUserSetting->route_allow_mass_status,
            allowEveScout: $this->mapUserSetting->route_use_evescout,
            map: $this->map,
            ignoredSystems: $ignored_systems,
            routePreference: $this->mapUserSetting->route_preference ?? RoutePreference::Shorter,
            securityPenalty: $this->mapUserSetting->security_penalty ?? 50,
        );

        return $this->routeService->findRoute($start_solarsystem_id, $destination_solarsystem_id, $options);
    }
}
