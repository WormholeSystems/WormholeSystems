<?php

declare(strict_types=1);

namespace App\Enums;

use App\Models\Wormhole;

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
    /**
     * The size dictated by an identified wormhole type. When it is known it
     * wins over every other source, because the hole's physics are not a
     * matter of preference.
     */
    public static function fromWormhole(?Wormhole $wormhole): ?self
    {
        return $wormhole instanceof Wormhole ? self::fromJumpMass($wormhole->maximum_jump_mass) : null;
    }

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
