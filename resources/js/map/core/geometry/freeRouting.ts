import type { EdgeGeometry, Rect, Vec2 } from '../types';

/** Padding of the endpoint "rail" inside each vertical edge of the node, in base units. */
export const RAIL_PADDING = 40;

/**
 * A connection endpoint that slides along a horizontal rail through the node's
 * centre. The rail runs at the vertical centre, inset RAIL_PADDING from each edge,
 * and the endpoint sits at the point on it nearest the other node (`towardX`) — so
 * it's pulled toward the far node but never closer than RAIL_PADDING to the edge.
 */
export function railEndpoint(box: Rect, towardX: number): Vec2 {
    const padding = Math.min(RAIL_PADDING, (box.maxX - box.minX) / 2);
    const x = Math.max(box.minX + padding, Math.min(towardX, box.maxX - padding));
    return { x, y: box.centerY };
}

/**
 * Free-layout routing for one edge: rail endpoints when both nodes are measured,
 * otherwise a straight fallback between whatever is known (a measured node's
 * centre, or the raw anchor until the size arrives).
 */
export function freeEdgeGeometry(id: number, source: Rect | Vec2, target: Rect | Vec2): EdgeGeometry {
    const sourceBox = 'minX' in source ? source : null;
    const targetBox = 'minX' in target ? target : null;

    if (!sourceBox || !targetBox) {
        return {
            id,
            kind: 'curve',
            from: sourceBox ? { x: sourceBox.centerX, y: sourceBox.centerY } : (source as Vec2),
            to: targetBox ? { x: targetBox.centerX, y: targetBox.centerY } : (target as Vec2),
        };
    }

    return {
        id,
        kind: 'curve',
        from: railEndpoint(sourceBox, targetBox.centerX),
        to: railEndpoint(targetBox, sourceBox.centerX),
    };
}
