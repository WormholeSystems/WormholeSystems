<?php

declare(strict_types=1);

namespace App\Services\JumpRange;

use App\Models\Solarsystem;

/**
 * Computes light-year distances between solar systems from their SDE coordinates,
 * which are stored in meters.
 */
final readonly class JumpRangeCalculator
{
    public const float METERS_PER_LIGHTYEAR = 9_460_730_472_580_800.0;

    public function distanceLy(Solarsystem $from, Solarsystem $to): float
    {
        return sqrt(
            ($from->pos_x - $to->pos_x) ** 2
            + ($from->pos_y - $to->pos_y) ** 2
            + ($from->pos_z - $to->pos_z) ** 2
        ) / self::METERS_PER_LIGHTYEAR;
    }
}
