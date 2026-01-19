<?php

declare(strict_types=1);

namespace App\Features;

use App\Services\EveScoutService;
use Inertia\ProvidesInertiaProperties;
use Inertia\RenderContext;

final readonly class EveScoutConnectionsFeature implements ProvidesInertiaProperties
{
    public function __construct(private EveScoutService $eveScoutService) {}

    public function toInertiaProperties(RenderContext $context): array
    {
        return [
            'eve_scout_connections' => $this->getEveScoutConnections(...),
        ];
    }

    private function getEveScoutConnections(): array
    {
        $connections = $this->eveScoutService->getWormholeConnections();

        return $connections
            ->map(fn ($connection): array => [
                'in_system_id' => $connection->in_system_id,
                'out_system_id' => $connection->out_system_id,
                'in_signature' => $connection->in_signature,
                'out_signature' => $connection->out_signature,
                'wormhole_type' => $connection->wormhole_type,
                'life' => $connection->life,
                'mass' => $connection->mass,
                'remaining_hours' => $connection->remaining_hours,
                'created_at' => $connection->created_at,
                'jumps_from_selected' => null,
            ])
            ->values()
            ->all();
    }
}
