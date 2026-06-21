<?php

declare(strict_types=1);

namespace App\Features;

use App\Enums\Permission;
use App\Enums\RemovableCard;
use App\Http\Resources\SelectedMapSolarsystemResource;
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
    /**
     * @param  string[]  $hiddenCards
     */
    public function __construct(
        private Map $map,
        private ?User $user,
        private ?MapSolarsystem $solarsystem = null,
        private array $hiddenCards = [],
    ) {}

    public function toInertiaProperties(RenderContext $context): array
    {
        return [
            'selected_map_solarsystem' => $this->getSelectedSolarsystem(...),
        ];
    }

    /**
     * @throws Throwable
     */
    private function getSelectedSolarsystem(): ?JsonResource
    {
        if (! $this->solarsystem instanceof MapSolarsystem) {
            return null;
        }

        $isViewer = ! $this->user instanceof User || $this->map->getUserPermission($this->user) === Permission::Viewer;
        $loadAudits = ! in_array(RemovableCard::Audits->value, $this->hiddenCards);

        $mapSolarsystem = $this->map->mapSolarsystems()
            ->with('signatures', 'wormholes', 'details')
            ->when(! $isViewer && $loadAudits, fn ($query) => $query->with('details.audits', fn (Relation $query) => $query->latest()))
            ->findOrFail($this->solarsystem->id);

        $mapSolarsystem->details->hideNotes($isViewer);

        return $mapSolarsystem->toResource(SelectedMapSolarsystemResource::class);
    }
}
