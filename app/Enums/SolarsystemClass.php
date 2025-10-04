<?php

declare(strict_types=1);

namespace App\Enums;

enum SolarsystemClass: string
{
    case C1 = '1';
    case C2 = '2';
    case C3 = '3';
    case C4 = '4';
    case C5 = '5';
    case C6 = '6';
    case C12 = '12'; // Thera
    case C13 = '13'; // Shattered
    case C14 = '14'; // Sentinel (Drifter)
    case C15 = '15'; // Barbican (Drifter)
    case C16 = '16'; // Vidette (Drifter)
    case C17 = '17'; // Conflux (Drifter)
    case C18 = '18'; // Redoubt (Drifter)
    case H = 'h';   // HighSec
    case L = 'l';   // LowSec
    case N = 'n';   // NullSec
    case Pochven = 'p';
    case Unknown = 'unknown';

    /**
     * Get a human-readable label for the solarsystem class.
     */
    public function label(): string
    {
        return match ($this) {
            self::C1 => 'Class 1',
            self::C2 => 'Class 2',
            self::C3 => 'Class 3',
            self::C4 => 'Class 4',
            self::C5 => 'Class 5',
            self::C6 => 'Class 6',
            self::C12 => 'Class 12 (Thera)',
            self::C13 => 'Class 13 (Shattered)',
            self::C14 => 'Class 14 (Sentinel)',
            self::C15 => 'Class 15 (Barbican)',
            self::C16 => 'Class 16 (Vidette)',
            self::C17 => 'Class 17 (Conflux)',
            self::C18 => 'Class 18 (Redoubt)',
            self::H => 'High Security',
            self::L => 'Low Security',
            self::N => 'Null Security',
            self::Pochven => 'Pochven',
            self::Unknown => 'Unknown',
        };
    }

    /**
     * Check if this is a standard wormhole class (C1-C6).
     */
    public function isStandard(): bool
    {
        return in_array($this, [
            self::C1,
            self::C2,
            self::C3,
            self::C4,
            self::C5,
            self::C6,
        ], true);
    }

    /**
     * Check if this is a special wormhole class (C12-C18).
     */
    public function isSpecial(): bool
    {
        return in_array($this, [
            self::C12,
            self::C13,
            self::C14,
            self::C15,
            self::C16,
            self::C17,
            self::C18,
        ], true);
    }

    /**
     * Check if this is a drifter wormhole class (C14-C18).
     */
    public function isDrifter(): bool
    {
        return in_array($this, [
            self::C14,
            self::C15,
            self::C16,
            self::C17,
            self::C18,
        ], true);
    }

    /**
     * Check if this is known space (HS, LS, NS).
     */
    public function isKnownSpace(): bool
    {
        return in_array($this, [
            self::H,
            self::L,
            self::N,
        ], true);
    }

    /**
     * Check if this is wormhole space (C1-C18).
     */
    public function isWormholeSpace(): bool
    {
        if ($this->isStandard()) {
            return true;
        }

        return $this->isSpecial();
    }
}
