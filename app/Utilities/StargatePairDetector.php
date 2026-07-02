<?php

declare(strict_types=1);

namespace App\Utilities;

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
