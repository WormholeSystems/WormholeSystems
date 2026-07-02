import { ANCHOR_OFFSET, ITEM_HEIGHT } from '../coords';
import type { Vec2 } from '../types';

export type OrganizeOptions = {
    maxSize: Vec2;
    gridSize: number;
    /** Grid cells kept between stacked systems (defaults to 1). */
    spacing?: number;
};

/**
 * Stacks systems into a single column at the top-left corner of their current
 * spread, keeping the given order top to bottom. Returns one position per id,
 * clamped to the canvas like a drag would be.
 */
export function organizeIntoColumn(orderedIds: number[], currentPositions: ReadonlyMap<number, Vec2>, options: OrganizeOptions): Map<number, Vec2> {
    const spacing = options.spacing ?? 1;
    const step = spacing * options.gridSize + ITEM_HEIGHT;

    let first: Vec2 = { x: Infinity, y: Infinity };
    for (const id of orderedIds) {
        const position = currentPositions.get(id);
        if (!position) continue;
        first = { x: Math.min(first.x, position.x), y: Math.min(first.y, position.y) };
    }

    const positions = new Map<number, Vec2>();
    orderedIds.forEach((id, index) => {
        positions.set(id, {
            x: Math.max(ANCHOR_OFFSET.x, Math.min(first.x, options.maxSize.x - options.gridSize)),
            y: Math.max(ANCHOR_OFFSET.y, Math.min(first.y + index * step, options.maxSize.y - options.gridSize)),
        });
    });
    return positions;
}
