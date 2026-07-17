<?php

declare(strict_types=1);

namespace App\Enums;

enum ShipSize: string
{
    case Frigate = 'frigate';
    case Medium = 'medium';
    case Large = 'large';
    case ExtraLarge = 'xlarge';

    /**
     * The largest ship class that fits through a wormhole with the given
     * maximum jump mass, using the standard 5M / 62M / <2B / 2B+ kg tiers.
     * Unknown or zero masses resolve to null.
     *
     * Mirrored on the frontend in resources/js/lib/shipSize.ts.
     */
    public static function fromJumpMass(?float $maximum_jump_mass): ?self
    {
        return match (true) {
            $maximum_jump_mass === null, $maximum_jump_mass <= 0.0 => null,
            $maximum_jump_mass <= 5_000_000.0 => self::Frigate,
            $maximum_jump_mass <= 62_000_000.0 => self::Medium,
            $maximum_jump_mass < 2_000_000_000.0 => self::Large,
            default => self::ExtraLarge,
        };
    }
}
