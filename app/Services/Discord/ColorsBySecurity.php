<?php

declare(strict_types=1);

namespace App\Services\Discord;

trait ColorsBySecurity
{
    private const int COLOR_HIGHSEC = 0x2ECC71;

    private const int COLOR_LOWSEC = 0xE67E22;

    private const int COLOR_NULLSEC = 0xE74C3C;

    private function colorForSecurity(float $security): int
    {
        return match (true) {
            $security >= 0.45 => self::COLOR_HIGHSEC,
            $security > 0.0 => self::COLOR_LOWSEC,
            default => self::COLOR_NULLSEC,
        };
    }
}
