<?php

declare(strict_types=1);

namespace App\Collections;

use App\Models\MapSolarsystem;
use Illuminate\Database\Eloquent\Collection;

/**
 * @extends Collection<int,MapSolarsystem>
 */
final class MapSolarsystemCollection extends Collection
{
    public function hasElementAtPosition(
        float $position_x,
        float $position_y,
        float $height,
        float $width
    ): bool {
        return $this->contains(fn (MapSolarsystem $solarsystem): bool => $this->overlaps(
            $solarsystem->position_x,
            $solarsystem->position_y,
            $position_x,
            $position_y,
            $height,
            $width
        ));
    }

    private function overlaps(
        ?float $x1,
        ?float $y1,
        float $x2,
        float $y2,
        float $height,
        float $width
    ): bool {
        if ($x1 === null || $y1 === null) {
            return false;
        }

        $start_x_1 = $x1;
        $end_x_1 = $x1 + $width;
        $start_y_1 = $y1;
        $end_y_1 = $y1 + $height;
        $start_x_2 = $x2;
        $end_x_2 = $x2 + $width;
        $start_y_2 = $y2;
        $end_y_2 = $y2 + $height;

        $overlaps_horizontally = $start_x_1 < $end_x_2 && $end_x_1 > $start_x_2;
        $overlaps_vertically = $start_y_1 < $end_y_2 && $end_y_1 > $start_y_2;

        return $overlaps_horizontally && $overlaps_vertically;
    }
}
