<?php

declare(strict_types=1);

namespace App\Features;

use App\Http\Resources\SolarsystemResource;
use App\Models\Map;
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

final readonly class MapClosestSystemsFeature implements ProvidesInertiaProperties
{
    public function __construct(
        private Stringable $from_system,
        private Stringable $condition,
        private int $limit,
        private MapUserSetting $mapUserSetting,
        private Map $map,
        private RouteService $routeService,
    ) {}

    public function toInertiaProperties(RenderContext $context): array
    {
        return [
            'closest_systems' => fn (): array => [
                'results' => $this->getClosestSystems(),
                'from_system' => $this->getFromSystem(),
                'condition' => $this->condition->toString(),
                'limit' => $this->limit,
            ],
        ];
    }

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

        $options = new RouteOptions(
            allowEol: $this->mapUserSetting->route_allow_eol,
            massStatus: $this->mapUserSetting->route_allow_mass_status,
            allowEveScout: $this->mapUserSetting->route_use_evescout,
            map: $this->map,
            ignoredSystems: Session::get('ignored_systems', []),
            routePreference: $this->mapUserSetting->route_preference,
            securityPenalty: $this->mapUserSetting->security_penalty ?? 50,
        );

        return $this->routeService->findClosestSystems($fromSystem->id, $options, $conditionClosure, $this->limit);
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
            'highsec' => fn ($query) => $query->where('security', '>=', 0.5),
            'lowsec' => fn ($query) => $query->whereBetween('security', [0.1, 0.4]),
            'nullsec' => fn ($query) => $query->where('security', '<=', 0.0),
            default => null,
        };
    }
}
