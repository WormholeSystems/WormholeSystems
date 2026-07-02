<?php

declare(strict_types=1);

namespace App\Enums;

/**
 * Jump-drive capable ship classes and their jump ranges. Base ranges are doubled
 * at Jump Drive Calibration V (each level adds 20%).
 */
enum JumpShipType: string
{
    case Dreadnought = 'dreadnought';
    case Carrier = 'carrier';
    case ForceAuxiliary = 'force_auxiliary';
    case Supercarrier = 'supercarrier';
    case Titan = 'titan';
    case JumpFreighter = 'jump_freighter';
    case Rorqual = 'rorqual';
    case BlackOps = 'black_ops';

    public function baseRangeLy(): float
    {
        return match ($this) {
            self::Dreadnought, self::Carrier, self::ForceAuxiliary => 3.5,
            self::Supercarrier, self::Titan => 3.0,
            self::JumpFreighter, self::Rorqual => 5.0,
            self::BlackOps => 4.0,
        };
    }

    public function maxRangeLy(int $jdcLevel): float
    {
        return $this->baseRangeLy() * (1 + 0.2 * $jdcLevel);
    }

    public function label(): string
    {
        return match ($this) {
            self::Dreadnought => 'Dreadnought',
            self::Carrier => 'Carrier',
            self::ForceAuxiliary => 'Force Auxiliary',
            self::Supercarrier => 'Supercarrier',
            self::Titan => 'Titan',
            self::JumpFreighter => 'Jump Freighter',
            self::Rorqual => 'Rorqual',
            self::BlackOps => 'Black Ops',
        };
    }

    /**
     * A representative hull for Dotlan range links; all hulls of a class share the same range.
     */
    public function dotlanShipName(): string
    {
        return match ($this) {
            self::Dreadnought => 'Naglfar',
            self::Carrier => 'Archon',
            self::ForceAuxiliary => 'Apostle',
            self::Supercarrier => 'Nyx',
            self::Titan => 'Avatar',
            self::JumpFreighter => 'Rhea',
            self::Rorqual => 'Rorqual',
            self::BlackOps => 'Redeemer',
        };
    }
}
