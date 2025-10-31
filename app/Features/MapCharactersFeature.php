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
        $characters = Character::query()
            ->hasTokenWithTrackingScopes()
            ->with([
                'characterStatus',
                'corporation',
                'alliance',
                'faction',
            ])
            ->tap(new UserAllowedMapTracking($this->map))
            ->tap(new CharacterHasMapAccess($this->map, without_guests: true))
            ->tap(new CharacterIsOnline)
            ->get();

        if (! $this->selectedMapSolarsystem instanceof MapSolarsystem) {
            return $characters->toResourceCollection(CharacterResource::class);
        }

        // Build route requests for all characters
        $routeRequests = [];
        foreach ($characters as $character) {
            if ($character->characterStatus && $character->characterStatus->solarsystem_id) {
                $routeRequests[$character->id] = [
                    'from' => $this->selectedMapSolarsystem->solarsystem_id,
                    'to' => $character->characterStatus->solarsystem_id,
                ];
            }
        }

        // Calculate all routes in one batched operation
        $routes = [];
        if ($routeRequests !== []) {
            $routes = $this->routeService->findMultipleRoutes($routeRequests, $this->getRouteOptions());
        }

        // Attach routes to characters
        return $characters
            ->map(function (Character $character) use ($routes): Character {
                $character->route = $routes[$character->id] ?? [];

                return $character;
            })
            ->toResourceCollection(CharacterResource::class);
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
