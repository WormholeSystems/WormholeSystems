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
    case C19 = '19'; // Abyssal Deadspace (ADR01) + Void regions (VR-01..05)
    case C20 = '20'; // Abyssal Deadspace (ADR02)
    case C21 = '21'; // Abyssal Deadspace (ADR03)
    case C22 = '22'; // Abyssal Deadspace (ADR04)
    case C23 = '23'; // Abyssal Deadspace (ADR05)
    case H = 'h';   // HighSec
    case L = 'l';   // LowSec
    case N = 'n';   // NullSec
    case Pochven = 'p';
    case Unknown = 'unknown';

    /**
     * Resolve the security-derived known-space class from a security status.
     *
     * This is the single home for the high/low/null thresholds; every other
     * representation derives from here.
     */
    public static function fromSecurity(float $security): self
    {
        return match (true) {
            $security >= 0.5 => self::H,
            $security >= 0.1 => self::L,
            default => self::N,
        };
    }

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
            self::C19 => 'Class 19 (Abyssal)',
            self::C20 => 'Class 20 (Abyssal)',
            self::C21 => 'Class 21 (Abyssal)',
            self::C22 => 'Class 22 (Abyssal)',
            self::C23 => 'Class 23 (Abyssal)',
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
     * Check if this is a special wormhole class (C12-C18: Thera, Shattered, Drifters).
     *
     * Note: C19-C23 are Abyssal Deadspace / Void regions, not wormhole space, so
     * they are intentionally excluded here.
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

    /**
     * A compact badge label, e.g. "C3", "H", "N", "P".
     */
    public function shortLabel(): string
    {
        return match ($this) {
            self::H => 'H',
            self::L => 'L',
            self::N => 'N',
            self::Pochven => 'P',
            self::Unknown => '?',
            default => 'C'.$this->value,
        };
    }

    /**
     * The Tailwind colour token (e.g. "c1", "hs") used to render this class.
     *
     * Mirrored to the frontend so colours/labels live in a single source.
     */
    public function colorToken(): string
    {
        return match ($this) {
            self::C1 => 'c1',
            self::C2 => 'c2',
            self::C3 => 'c3',
            self::C4 => 'c4',
            self::C5 => 'c5',
            self::C6 => 'c6',
            self::C12 => 'c12',
            self::C13 => 'c13',
            self::C14 => 'c14',
            self::C15 => 'c15',
            self::C16 => 'c16',
            self::C17 => 'c17',
            self::C18 => 'c18',
            self::C19, self::C20, self::C21, self::C22, self::C23 => 'unknown',
            self::H => 'hs',
            self::L => 'ls',
            self::N => 'ns',
            self::Pochven => 'pochven',
            self::Unknown => 'unknown',
        };
    }

    /**
     * A stable weight for ordering systems by class: known space first
     * (high, low, null, pochven), then wormhole space by class number.
     */
    public function sortWeight(): int
    {
        return match ($this) {
            self::H => 0,
            self::L => 1,
            self::N => 2,
            self::Pochven => 3,
            self::Unknown => 99,
            default => 10 + (int) $this->value,
        };
    }
}
