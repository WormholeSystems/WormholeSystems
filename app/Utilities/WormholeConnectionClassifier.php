<?php

namespace App\Utilities;

use App\Enums\ShipSize;
use App\Models\Solarsystem;

class WormholeConnectionClassifier
{
    public function getSize(Solarsystem $from, Solarsystem $to): ?ShipSize
    {
        // @see /resources/js/hooks/useNewConnection.ts

        $classes = collect([$from->wormholeSystem?->class, $to->wormholeSystem?->class])
            ->filter(fn ($c): bool => $c !== null);

        if ($classes->contains(fn ($c): bool => $c >= 13 && $c <= 18)) {
            return ShipSize::Frigate;
        }

        if ($classes->contains(1)) {
            return ShipSize::Medium;
        }

        // Check if Turnur connects to JSpace
        $names = collect([$from->name, $to->name]);
        if ($names->contains('Turnur') && $classes->isNotEmpty()) {
            return ShipSize::Medium;
        }

        // Check if Thera connects to Highsec
        $highsec = collect([$from->security, $to->security])
            ->filter(fn ($s): bool => $s >= 0.5);
        if ($names->contains('Thera') && $highsec->isNotEmpty()) {
            return ShipSize::Medium;
        }

        return null;
    }
}
