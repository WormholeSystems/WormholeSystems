<?php

declare(strict_types=1);

namespace App\Features;

use App\Enums\RoutePreference;
use App\Http\Resources\CharacterResource;
use App\Models\Character;
use App\Models\Map;
use App\Models\MapSolarsystem;
use App\Models\MapUserSetting;
use App\Scopes\CharacterHasMapAccess;
use App\Scopes\CharacterIsOnline;
use App\Scopes\UserAllowedMapTracking;
use App\Services\RouteOptions;
use App\Services\RouteService;
use Illuminate\Http\Resources\Json\ResourceCollection;
use Illuminate\Support\Facades\Session;
use Inertia\ProvidesInertiaProperties;
use Inertia\RenderContext;
use Throwable;

final readonly class MapCharactersFeature implements ProvidesInertiaProperties
{
    public function __construct(
        private Map $map,
        private bool $canViewCharacters,
        private ?MapSolarsystem $selectedMapSolarsystem,
        private MapUserSetting $mapUserSetting,
        private RouteService $routeService,
    ) {}

    public function toInertiaProperties(RenderContext $context): array
    {
        return [
            'map_characters' => fn (): ?ResourceCollection => $this->canViewCharacters ? $this->getMapCharacters() : null,
        ];
    }

    /**
     * @throws Throwable
     */
    private function getMapCharacters(): ResourceCollection
    {
        return Character::query()
            ->hasTokenWithTrackingScopes()
            ->with('characterStatus')
            ->tap(new UserAllowedMapTracking($this->map))
            ->tap(new CharacterHasMapAccess($this->map, without_guests: true))
            ->tap(new CharacterIsOnline)
            ->get()
            ->map(function (Character $character): Character {
                if (! $this->selectedMapSolarsystem instanceof MapSolarsystem) {
                    return $character;
                }

                $character->route = $this->getFastestRoute(
                    $this->selectedMapSolarsystem->solarsystem_id ?? 0,
                    $character->characterStatus->solarsystem_id ?? 0
                );

                return $character;
            })
            ->toResourceCollection(CharacterResource::class);
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
