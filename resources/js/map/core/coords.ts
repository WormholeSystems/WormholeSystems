import type { Rect, Size, Vec2 } from './types';

/**
 * The single boundary between the two coordinate spaces: everything in the map
 * subsystem (store, geometry, interactions) works in base units; scale is applied
 * only where coordinates are emitted into the DOM/SVG, and screen input is
 * converted back the moment it enters.
 */

/**
 * Where a node's connection anchor (its stored position) sits relative to the
 * node's top-left, in base units. Nodes render translated by this much so the
 * anchor lands at the right spot; edge routing uses it to reconstruct the box.
 */
export const ANCHOR_OFFSET: Vec2 = { x: 40, y: 20 };

/** Height of a plain node card without a pilot row, in base units. */
export const ITEM_HEIGHT = 40;

export function scalePoint(point: Vec2, scale: number): Vec2 {
    return { x: point.x * scale, y: point.y * scale };
}

export function scaleLength(value: number, scale: number): number {
    return value * scale;
}

export function snapToGrid(point: Vec2, gridSize: number): Vec2 {
    return {
        x: Math.round(point.x / gridSize) * gridSize,
        y: Math.round(point.y / gridSize) * gridSize,
    };
}

/** Keeps an anchor inside the canvas: no closer to the edges than the node overhang. */
export function clampToCanvas(point: Vec2, maxSize: Vec2, gridSize: number): Vec2 {
    return {
        x: Math.max(ANCHOR_OFFSET.x, Math.min(point.x, maxSize.x - gridSize)),
        y: Math.max(ANCHOR_OFFSET.y, Math.min(point.y, maxSize.y - gridSize)),
    };
}

/** The node bounding box reconstructed from its anchor and measured size. */
export function nodeRect(anchor: Vec2, size: Size): Rect {
    const minX = anchor.x - ANCHOR_OFFSET.x;
    const minY = anchor.y - ANCHOR_OFFSET.y;
    const maxX = minX + size.width;
    const maxY = minY + size.height;
    return { minX, minY, maxX, maxY, centerX: (minX + maxX) / 2, centerY: (minY + maxY) / 2 };
}

/**
 * The viewport measurements needed to convert screen input to base units,
 * captured once at gesture start instead of per pointer event.
 */
export type ViewportFrame = {
    rectLeft: number;
    rectTop: number;
    scrollLeft: number;
    scrollTop: number;
    scale: number;
};

export function clientToBase(clientX: number, clientY: number, frame: ViewportFrame): Vec2 {
    return {
        x: (clientX - frame.rectLeft + frame.scrollLeft) / frame.scale,
        y: (clientY - frame.rectTop + frame.scrollTop) / frame.scale,
    };
}
