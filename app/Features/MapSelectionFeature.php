<?php

declare(strict_types=1);

namespace App\Features;

use App\Enums\Permission;
use App\Http\Resources\MapSolarsystemResource;
use App\Models\Map;
use App\Models\MapSolarsystem;
use App\Models\User;
use Illuminate\Database\Eloquent\Relations\Relation;
use Illuminate\Http\Resources\Json\JsonResource;
use Inertia\ProvidesInertiaProperties;
use Inertia\RenderContext;
use Throwable;

final readonly class MapSelectionFeature implements ProvidesInertiaProperties
{
    private ?MapSolarsystem $selectedSolarsystem;

    public function __construct(
        private Map $map,
        private User $user,
        private ?int $selected_map_solarsystem_id,
    ) {
        $this->selectedSolarsystem = $this->getSelectedSolarsystem();
    }

    public function toInertiaProperties(RenderContext $context): array
    {
        return [
            'selected_map_solarsystem' => fn (): ?JsonResource => $this->selectedSolarsystem?->toResource(MapSolarsystemResource::class),
        ];
    }

    public function getSelectedMapSolarsystem(): ?MapSolarsystem
    {
        return $this->selectedSolarsystem;
    }

    /**
     * @throws Throwable
     */
    private function getSelectedSolarsystem(): ?MapSolarsystem
    {
        if ($this->selected_map_solarsystem_id === null || $this->selected_map_solarsystem_id === 0) {
            return null;
        }

        $isGuest = $this->map->getUserPermission($this->user) === Permission::Guest;

        return $this->map->mapSolarsystems()
            ->with('signatures', 'wormholes')
            ->when(! $isGuest, fn ($query) => $query->with('audits', fn (Relation $query) => $query->latest()))
            ->findOrFail($this->selected_map_solarsystem_id)
            ->hideNotes($isGuest);
    }
}
