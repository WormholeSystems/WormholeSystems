<?php

declare(strict_types=1);

namespace App\Features;

use App\Enums\SignatureCategory;
use App\Http\Resources\MapSolarsystemResource;
use App\Models\Map;
use App\Models\MapSolarsystem;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Relations\Relation;
use Illuminate\Http\Resources\Json\JsonResource;
use Inertia\Inertia;
use Inertia\ProvidesInertiaProperties;
use Inertia\RenderContext;
use Throwable;

final readonly class MapTrackingFeature implements ProvidesInertiaProperties
{
    public function __construct(
        private Map $map,
        private ?int $origin_map_solarsystem_id,
        private ?int $target_solarsystem_id,
    ) {}

    public function toInertiaProperties(RenderContext $context): array
    {
        return [
            'tracking_origin' => Inertia::optional(fn (): ?JsonResource => $this->getTrackingOrigin()),
            'tracking_target' => Inertia::optional(fn (): ?array => $this->getTrackingTarget()),
        ];
    }

    /**
     * Get signatures for the tracking origin solarsystem
     *
     * @throws Throwable
     */
    private function getTrackingOrigin(): ?JsonResource
    {
        if (! $this->origin_map_solarsystem_id) {
            return null;
        }

        $solarsystem = $this->map->mapSolarsystems()->find($this->origin_map_solarsystem_id);

        if (! $solarsystem instanceof MapSolarsystem) {
            return null;
        }

        $solarsystem->load([
            'signatures' => fn (Relation $query) => $query
                ->where('map_solarsystem_id', $this->origin_map_solarsystem_id)
                ->where(fn (Builder $q) => $q
                    ->whereRelation('signatureCategory', 'code', SignatureCategory::Wormhole)
                    ->orWhereNull('signature_category_id'))
                ->with(['signatureType', 'signatureCategory', 'wormhole', 'mapConnection']),
        ])
            ->loadCount('signatures', 'wormholeSignatures', 'mapConnections', 'uncategorizedSignatures');

        return $solarsystem->toResource(MapSolarsystemResource::class);
    }

    /**
     * Get the target solarsystem for tracking
     *
     * @throws Throwable
     */
    private function getTrackingTarget(): ?array
    {
        if (! $this->target_solarsystem_id) {
            return null;
        }

        return ['solarsystem_id' => $this->target_solarsystem_id];
    }
}
