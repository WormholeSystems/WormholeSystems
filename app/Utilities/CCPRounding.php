<?php

declare(strict_types=1);

namespace App\Utilities;

/**
 * CCPRounding
 *
 * PHP version of Nohus' CCPRounding class.
 *
 * https://gitlab.com/rift-intel-fusion-tool/
 */
final class CCPRounding
{
    public static function roundSecurity(float $number): float
    {
        if ($number === 0.0) {
            return 0.0;
        }

        if ($number >= 0.0 && $number < 0.05) {
            return round($number * 10 + 0.5) / 10.0;
        }

        return round($number * 10) / 10.0;
    }
}
