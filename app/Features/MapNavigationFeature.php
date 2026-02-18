<?php

declare(strict_types=1);

namespace App\Features;

use App\Enums\RemovableCard;
use App\Models\Map;
use Inertia\ProvidesInertiaProperties;
use Inertia\RenderContext;

final readonly class MapNavigationFeature implements ProvidesInertiaProperties
{
    /**
     * @param  string[]  $hiddenCards
     */
    public function __construct(
        private Map $map,
        private array $hiddenCards = [],
    ) {}

    public function toInertiaProperties(RenderContext $context): array
    {
        if (in_array(RemovableCard::Autopilot->value, $this->hiddenCards)) {
            return [];
        }

        return [
            'map_navigation' => fn (): array => [
                'destinations' => $this->getDestinations(),
            ],
        ];
    }

    /**
     * Get the map's destinations (watchlist systems) for the client to build routes against.
     *
     * @return array<int, array{id: int, map_id: int, solarsystem_id: int, is_pinned: bool}>
     */
    private function getDestinations(): array
    {
        return $this->map->mapRouteSolarsystems
            ->map(fn ($mapRouteSolarsystem): array => [
                'id' => $mapRouteSolarsystem->id,
                'map_id' => $mapRouteSolarsystem->map_id,
                'solarsystem_id' => $mapRouteSolarsystem->solarsystem_id,
                'is_pinned' => $mapRouteSolarsystem->is_pinned,
            ])
            ->all();
    }
}
