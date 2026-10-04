<?php

declare(strict_types=1);

namespace App\Utilities;

use App\Enums\ConnectionType;
use App\Models\MapSolarsystem;
use App\Models\Solarsystem;
use NicolasKion\SDE\Models\SolarsystemConnection;

final class StargatePairDetector
{
    /**
     * Whether two systems are k-space neighbors linked by a stargate,
     * meaning a jump between them is not a wormhole transit.
     */
    public function isStargatePair(Solarsystem $from, Solarsystem $to): bool
    {
        return $this->isKSpaceToKSpaceConnection($from, $to)
            && $this->systemsAreConnectedPerStargates($from, $to);
    }

    /**
     * The default type for a connection drawn between two map systems:
     * a stargate when they are gate neighbors, a wormhole otherwise.
     */
    public function connectionTypeBetween(int $from_map_solarsystem_id, int $to_map_solarsystem_id): ConnectionType
    {
        $map_solarsystems = MapSolarsystem::query()
            ->with('solarsystem')
            ->findMany([$from_map_solarsystem_id, $to_map_solarsystem_id])
            ->keyBy('id');

        $from = $map_solarsystems->get($from_map_solarsystem_id)?->solarsystem;
        $to = $map_solarsystems->get($to_map_solarsystem_id)?->solarsystem;

        if ($from instanceof Solarsystem && $to instanceof Solarsystem && $this->isStargatePair($from, $to)) {
            return ConnectionType::Stargate;
        }

        return ConnectionType::Wormhole;
    }

    private function isKSpaceToKSpaceConnection(Solarsystem $from, Solarsystem $to): bool
    {
        return $from->type === 'eve' && $to->type === 'eve';
    }

    private function systemsAreConnectedPerStargates(Solarsystem $from, Solarsystem $to): bool
    {
        return SolarsystemConnection::query()
            ->where('from_solarsystem_id', $from->id)
            ->where('to_solarsystem_id', $to->id)
            ->exists();
    }
}
