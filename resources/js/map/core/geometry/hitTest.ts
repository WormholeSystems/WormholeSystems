import type { Vec2 } from '../types';

export type Box = { start: Vec2; end: Vec2 };

/**
 * The ids of systems whose anchor lies inside the (unordered) marquee box.
 * Matches the sticky-selection semantics: callers commit the result as the box
 * is drawn, so dragging systems out of the box later doesn't deselect them.
 */
export function systemsInBox(positions: Iterable<[number, Vec2]>, box: Box): number[] {
    const minX = Math.min(box.start.x, box.end.x);
    const maxX = Math.max(box.start.x, box.end.x);
    const minY = Math.min(box.start.y, box.end.y);
    const maxY = Math.max(box.start.y, box.end.y);

    const ids: number[] = [];
    for (const [id, position] of positions) {
        if (position.x >= minX && position.x <= maxX && position.y >= minY && position.y <= maxY) {
            ids.push(id);
        }
    }
    return ids;
}
