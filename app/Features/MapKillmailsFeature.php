<?php

declare(strict_types=1);

namespace App\Features;

use App\Enums\KillmailFilter;
use App\Http\Resources\KillmailResource;
use App\Models\Killmail;
use App\Models\Map;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Http\Resources\Json\ResourceCollection;
use Inertia\ProvidesInertiaProperties;
use Inertia\RenderContext;
use Throwable;

final readonly class MapKillmailsFeature implements ProvidesInertiaProperties
{
    public function __construct(
        private Map $map,
        private KillmailFilter $filter,
    ) {}

    public function toInertiaProperties(RenderContext $context): array
    {
        return [
            'map_killmails' => $this->getMapKills(...),
        ];
    }

    /**
     * @throws Throwable
     */
    private function getMapKills(): ResourceCollection
    {
        return Killmail::query()->with('shipType')
            ->whereIn('solarsystem_id', $this->map->mapSolarsystems->pluck('solarsystem_id'))
            ->when($this->filter === KillmailFilter::KSpace, fn (Builder $query) => $query->whereRelation('solarsystem', 'type', 'eve'))
            ->when($this->filter === KillmailFilter::JSpace, fn (Builder $query) => $query->whereRelation('solarsystem', 'type', 'wormhole'))
            ->orderByDesc('id')
            ->limit(50)
            ->get()
            ->toResourceCollection(KillmailResource::class);
    }
}
