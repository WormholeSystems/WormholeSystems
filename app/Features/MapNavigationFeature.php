<?php

declare(strict_types=1);

namespace App\Features;

use App\Enums\Permission;
use App\Enums\RemovableCard;
use App\Models\Map;
use App\Models\MapRouteSolarsystem;
use App\Models\User;
use Inertia\ProvidesInertiaProperties;
use Inertia\RenderContext;

final readonly class MapNavigationFeature implements ProvidesInertiaProperties
{
    /**
     * @param  string[]  $hiddenCards
     */
    public function __construct(
        private Map $map,
        private ?User $user,
        private array $hiddenCards = [],
    ) {}

    public function toInertiaProperties(RenderContext $context): array
    {
        if (in_array(RemovableCard::Autopilot->value, $this->hiddenCards)) {
            return [];
        }

        return [
            'map_navigation' => function (): array {
                $canAddPersonal = $this->canAddPersonal();
                $canAddShared = $this->user instanceof User && $this->user->can('updateSettings', $this->map);

                return [
                    'destinations' => $this->getDestinations($canAddPersonal, $canAddShared),
                    'can_add_personal' => $canAddPersonal,
                    'can_add_shared' => $canAddShared,
                ];
            },
        ];
    }

    /**
     * Only users with an accessor row get a personal watchlist: anonymous, share-token and
     * public-map visitors only see the shared one.
     */
    private function canAddPersonal(): bool
    {
        return $this->user instanceof User && $this->map->getUserPermission($this->user) instanceof Permission;
    }

    /**
     * Get the map's shared destinations plus the user's personal ones (watchlist systems) for
     * the client to build routes against. A system on both lists is returned twice.
     *
     * @return array<int, array{id: int, map_id: int, solarsystem_id: int, is_pinned: bool, is_personal: bool, can_edit: bool}>
     */
    private function getDestinations(bool $canEditPersonal, bool $canEditShared): array
    {
        return MapRouteSolarsystem::query()
            ->where('map_id', $this->map->id)
            ->visibleTo($canEditPersonal ? $this->user : null)
            ->orderBy('id')
            ->get()
            ->map(fn (MapRouteSolarsystem $mapRouteSolarsystem): array => [
                'id' => $mapRouteSolarsystem->id,
                'map_id' => $mapRouteSolarsystem->map_id,
                'solarsystem_id' => $mapRouteSolarsystem->solarsystem_id,
                'is_pinned' => $mapRouteSolarsystem->is_pinned,
                'is_personal' => ! $mapRouteSolarsystem->isShared(),
                'can_edit' => $mapRouteSolarsystem->isShared() ? $canEditShared : $canEditPersonal,
            ])
            ->all();
    }
}
