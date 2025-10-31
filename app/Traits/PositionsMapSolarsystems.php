<?php

declare(strict_types=1);

namespace App\Traits;

use App\Models\Map;
use App\Models\MapSolarsystem;
use Random\RandomException;

use function range;

/**
 * Trait to manage positions of solarsystems on a map in a column pattern
 *
 * @property-read int $grid_size
 * @property-read int $max_x
 * @property-read int $max_y
 */
trait PositionsMapSolarsystems
{
    /**
     * Get next free position in a column pattern
     *
     * @return array{position_x: int, position_y: int}
     *
     * @throws RandomException
     */
    private function getNextFreePosition(
        Map $map,
        MapSolarsystem $referenceSystem,
        int $minDistanceY,
        int $minDistanceX,
        int $maxTries = 100
    ): array {
        $occupiedPositions = $map->mapSolarsystems()
            ->isOnMap()
            ->get(['position_x', 'position_y', 'id']);

        $new_y = $referenceSystem->position_y;
        $new_x = $referenceSystem->position_x;

        foreach (range(0, $maxTries) as $ignored) {
            $new_y += $minDistanceY;

            // If we exceed max Y, wrap to next column
            if ($new_y > $this->max_y) {
                $new_y = $minDistanceY;
                $new_x += $minDistanceX;
            }

            if (! $occupiedPositions->hasElementAtPosition($new_x, $new_y, $minDistanceY, $minDistanceX)) {
                return $this->constrainPositionToMapBounds($new_x, $new_y);
            }
        }

        // If we couldn't find a free position after many tries, use random position around reference
        return $this->getRandomPositionAroundSystem($referenceSystem, $minDistanceX, $minDistanceY);
    }

    /**
     * @return array{position_x: int, position_y: int}
     *
     * @throws RandomException
     */
    private function getRandomPositionAroundSystem(
        MapSolarsystem $referenceSystem,
        int $minDistanceX,
        int $minDistanceY
    ): array {
        $distance_x = random_int($minDistanceX, $minDistanceX * 2);
        $distance_y = random_int($minDistanceY, $minDistanceY * 2);
        $direction_y = random_int(0, 1) !== 0 ? 1 : -1;

        $position_x = $referenceSystem->position_x + $distance_x;
        $position_y = $referenceSystem->position_y + ($distance_y * $direction_y);

        return $this->constrainPositionToMapBounds($position_x, $position_y);
    }

    /**
     * @return array{position_x: int, position_y: int}
     */
    private function constrainPositionToMapBounds(int $position_x, int $position_y): array
    {
        // Snap to grid
        $position_x = (int) (round($position_x / $this->grid_size) * $this->grid_size);
        $position_y = (int) (round($position_y / $this->grid_size) * $this->grid_size);

        // Constrain to map bounds
        $position_x = max(40, min($this->max_x, $position_x));
        $position_y = max(20, min($this->max_y, $position_y));

        return [
            'position_x' => $position_x,
            'position_y' => $position_y,
        ];
    }
}
