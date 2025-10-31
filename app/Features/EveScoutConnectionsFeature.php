<?php

declare(strict_types=1);

namespace App\Features;

use App\Http\Resources\SolarsystemResource;
use App\Models\Map;
use App\Models\MapSolarsystem;
use App\Models\Solarsystem;
use App\Services\EveScoutService;
use App\Services\RouteOptions;
use App\Services\RouteService;
use Inertia\ProvidesInertiaProperties;
use Inertia\RenderContext;
use Throwable;

final readonly class EveScoutConnectionsFeature implements ProvidesInertiaProperties
{
    public function __construct(
        private EveScoutService $eveScoutService,
        private RouteService $routeService,
        private Map $map,
        private ?MapSolarsystem $selectedSolarsystem,
    ) {}

    public function toInertiaProperties(RenderContext $context): array
    {
        return [
            'eve_scout_connections' => $this->getEveScoutConnections(),
        ];
    }

    /**
     * Get EVE Scout connections (Thera, Turnur, etc.) with enriched solarsystem data
     *
     * @return array<array{in_system: array, out_system: array, in_signature: string, out_signature: string, wormhole_type: string, life: string, mass: string, remaining_hours: float|null, created_at: string|null, jumps_from_selected: int|null}>
     */
    private function getEveScoutConnections(): array
    {
        $connections = $this->eveScoutService->getWormholeConnections();

        $system_ids = $connections->flatMap(fn ($conn): array => [$conn->in_system_id, $conn->out_system_id])->unique();

        // Fetch all solarsystems with their relationships
        $solarsystems = Solarsystem::query()
            ->with([
                'sovereignty' => ['alliance', 'corporation', 'faction'],
                'wormholeSystem.effect',
                'constellation',
                'region',
            ])
            ->whereIn('id', $system_ids)
            ->get()
            ->keyBy('id');

        // Calculate jumps from selected system if available
        $jumpsMap = [];
        if ($this->selectedSolarsystem instanceof MapSolarsystem) {
            $fromSystemId = $this->selectedSolarsystem->solarsystem_id;
            $routeOptions = new RouteOptions(
                allowEol: false,
                allowEveScout: false, // Don't use EVE Scout connections for jump calculation
                map: $this->map,
            );

            // Calculate jumps to each unique system (excluding Thera and Turnur)
            $theraId = Solarsystem::query()->where('name', 'Thera')->value('id');
            $turnurId = Solarsystem::query()->where('name', 'Turnur')->value('id');

            foreach ($system_ids as $systemId) {
                if ($systemId === $theraId) {
                    continue;
                }
                if ($systemId === $turnurId) {
                    continue;
                }
                try {
                    $route = $this->routeService->findRoute($fromSystemId, $systemId, $routeOptions);
                    // Route includes origin, so jumps = count - 1
                    $jumpsMap[$systemId] = $route !== [] ? count($route) - 1 : null;
                } catch (Throwable) {
                    $jumpsMap[$systemId] = null;
                }
            }
        }

        // Enrich connections with full solarsystem data and jumps
        return collect($connections)
            ->map(function ($connection) use ($solarsystems, $jumpsMap): ?array {
                $inSystem = $solarsystems->get($connection->in_system_id);
                $outSystem = $solarsystems->get($connection->out_system_id);

                // Skip if either system is not found
                if (! $inSystem instanceof Solarsystem || ! $outSystem instanceof Solarsystem) {
                    return null;
                }

                // Determine which system is Thera/Turnur and calculate jumps to the other
                $jumpsFromSelected = null;
                if ($jumpsMap !== []) {
                    // If in_system is Thera/Turnur, get jumps to out_system
                    if (in_array($inSystem->name, ['Thera', 'Turnur'])) {
                        $jumpsFromSelected = $jumpsMap[$outSystem->id] ?? null;
                    } else {
                        $jumpsFromSelected = $jumpsMap[$inSystem->id] ?? null;
                    }
                }

                return [
                    'in_system' => $inSystem->toResource(SolarsystemResource::class),
                    'out_system' => $outSystem->toResource(SolarsystemResource::class),
                    'in_signature' => $connection->in_signature,
                    'out_signature' => $connection->out_signature,
                    'wormhole_type' => $connection->wormhole_type,
                    'life' => $connection->life,
                    'mass' => $connection->mass,
                    'remaining_hours' => $connection->remaining_hours,
                    'created_at' => $connection->created_at,
                    'jumps_from_selected' => $jumpsFromSelected,
                ];
            })
            ->filter()
            ->values()
            ->toArray();
    }
}
