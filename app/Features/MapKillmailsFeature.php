<?php

declare(strict_types=1);

namespace App\Features;

use App\Enums\KillmailFilter;
use App\Enums\RemovableCard;
use App\Http\Resources\KillmailResource;
use App\Models\Killmail;
use App\Models\Map;
use App\Models\Solarsystem;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Http\Resources\Json\ResourceCollection;
use Illuminate\Support\Collection;
use Inertia\Inertia;
use Inertia\ProvidesInertiaProperties;
use Inertia\RenderContext;
use Throwable;

final readonly class MapKillmailsFeature implements ProvidesInertiaProperties
{
    private const int LIMIT = 50;

    /**
     * @param  string[]  $hiddenCards
     */
    public function __construct(
        private Map $map,
        private KillmailFilter $filter,
        private ?int $selectedSolarsystemId = null,
        private array $hiddenCards = [],
    ) {}

    public function toInertiaProperties(RenderContext $context): array
    {
        if (in_array(RemovableCard::Killmails->value, $this->hiddenCards)) {
            return [];
        }

        return [
            'map_killmails' => Inertia::defer($this->getMapKills(...)),
        ];
    }

    /**
     * @throws Throwable
     */
    private function getMapKills(): ResourceCollection
    {
        if ($this->filter === KillmailFilter::SelectedSystem && $this->selectedSolarsystemId === null) {
            return collect()->toResourceCollection(KillmailResource::class);
        }

        return Killmail::query()
            ->with([
                'shipType',
                'victimCorporation:id,name,ticker',
                'victimAlliance:id,name,ticker',
            ])
            ->whereIn('id', $this->latestKillmailIds())
            ->orderByDesc('id')
            ->get()
            ->toResourceCollection(KillmailResource::class);
    }

    /**
     * One query over all systems lets MySQL sort every kill of a busy system
     * (or walk the primary key past unrelated kills) before taking fifty.
     * Taking the newest fifty per system off the solarsystem index keeps the
     * work bounded by the number of systems on the map.
     *
     * @return Collection<int, int>
     */
    private function latestKillmailIds(): Collection
    {
        $solarsystem_ids = $this->filteredSolarsystemIds();

        if ($solarsystem_ids->isEmpty()) {
            return collect();
        }

        $query = $solarsystem_ids
            ->map(fn (int $solarsystem_id): Builder => Killmail::query()
                ->select('id')
                ->where('solarsystem_id', $solarsystem_id)
                ->orderByDesc('id')
                ->limit(self::LIMIT))
            ->reduce(fn (?Builder $union, Builder $query): Builder => $union instanceof Builder ? $union->unionAll($query) : $query);

        return $query->orderByDesc('id')->limit(self::LIMIT)->pluck('id');
    }

    /**
     * @return Collection<int, int>
     */
    private function filteredSolarsystemIds(): Collection
    {
        if ($this->filter === KillmailFilter::SelectedSystem) {
            return $this->selectedSolarsystemId === null ? collect() : collect([$this->selectedSolarsystemId]);
        }

        $solarsystem_ids = $this->map->mapSolarsystems->pluck('solarsystem_id')->unique()->values();

        $type = match ($this->filter) {
            KillmailFilter::All => null,
            KillmailFilter::KSpace => 'eve',
            KillmailFilter::JSpace => 'wh',
        };

        if ($type === null || $solarsystem_ids->isEmpty()) {
            return $solarsystem_ids;
        }

        return Solarsystem::query()
            ->whereIn('id', $solarsystem_ids)
            ->where('type', $type)
            ->pluck('id');
    }
}
