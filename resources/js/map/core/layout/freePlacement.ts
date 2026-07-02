import type { Vec2 } from '../types';

export type FreePlacementOptions = {
    maxSize: Vec2;
    gridSize: number;
    /** Distance kept from the canvas edges. */
    padding?: number;
};

/**
 * The area around a candidate anchor that must be clear of other anchors: wider to
 * the right (where the node body extends) than to the left/top, in base units.
 */
const CLEARANCE = { x1: -30, y1: -30, x2: 80, y2: 30 };

/**
 * The first grid cell, scanning column by column from the top-left, whose
 * clearance box contains no occupied anchor. Throws when the canvas is full.
 */
export function findFreePosition(occupied: Vec2[], options: FreePlacementOptions): Vec2 {
    const padding = options.padding ?? 100;
    const { gridSize, maxSize } = options;

    for (let x = padding; x < maxSize.x - padding; x += gridSize) {
        for (let y = padding; y < maxSize.y - padding; y += gridSize) {
            const overlaps = occupied.some(
                (anchor) =>
                    anchor.x >= x + CLEARANCE.x1 && anchor.x <= x + CLEARANCE.x2 && anchor.y >= y + CLEARANCE.y1 && anchor.y <= y + CLEARANCE.y2,
            );
            if (!overlaps) {
                return { x, y };
            }
        }
    }

    throw new Error('No free position found on the map');
}
