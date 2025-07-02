<?php

namespace App\Utilities;

/**
 * CCPRounding
 *
 * PHP version of Nohus' CCPRounding class.
 *
 * https://gitlab.com/rift-intel-fusion-tool/
 */
class CCPRounding
{
    public static function roundSecurity(float $number): float
    {
        if ($number === 0.0) {
            return 0.0;
        }

        if ($number >= 0.0 && $number < 0.05) {
            return ((int) ($number * 10 + 0.5)) / 10.0;
        }

        return ((int) ($number * 10)) / 10.0;
    }
}
