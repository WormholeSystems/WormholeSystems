import { ANCHOR_OFFSET, clampToCanvas, clientToBase, nodeRect, scalePoint, snapToGrid, type ViewportFrame } from '@/map/core/coords';
import { systemsInBox } from '@/map/core/geometry/hitTest';
import { findFreePosition } from '@/map/core/layout/freePlacement';
import { organizeIntoColumn } from '@/map/core/layout/organize';
import { describe, expect, it } from 'vitest';

const MAX_SIZE = { x: 4000, y: 2000 };
const GRID = 20;

describe('snapToGrid / clampToCanvas', () => {
    it('matches the drag math: snap to the grid, clamp to the anchor overhang and far edge', () => {
        expect(snapToGrid({ x: 31, y: 49 }, GRID)).toEqual({ x: 40, y: 40 });
        // Ports the old handleDrag clamp: x within [40, max - grid], y within [20, max - grid].
        expect(clampToCanvas({ x: 0, y: 0 }, MAX_SIZE, GRID)).toEqual({ x: ANCHOR_OFFSET.x, y: ANCHOR_OFFSET.y });
        expect(clampToCanvas({ x: 99999, y: 99999 }, MAX_SIZE, GRID)).toEqual({ x: MAX_SIZE.x - GRID, y: MAX_SIZE.y - GRID });
        expect(clampToCanvas({ x: 500, y: 500 }, MAX_SIZE, GRID)).toEqual({ x: 500, y: 500 });
    });
});

describe('clientToBase', () => {
    it.each([0.5, 1, 2])('round-trips with scalePoint at scale %s', (scale) => {
        const frame: ViewportFrame = { rectLeft: 13, rectTop: 7, scrollLeft: 250, scrollTop: 40, scale };
        const base = clientToBase(500, 300, frame);
        const backOnScreen = scalePoint(base, scale);
        expect(backOnScreen.x).toBeCloseTo(500 - frame.rectLeft + frame.scrollLeft);
        expect(backOnScreen.y).toBeCloseTo(300 - frame.rectTop + frame.scrollTop);
    });
});

describe('nodeRect', () => {
    it('offsets the box by the anchor offset and precomputes the centre', () => {
        const rect = nodeRect({ x: 100, y: 100 }, { width: 180, height: 40 });
        expect(rect).toEqual({ minX: 60, minY: 80, maxX: 240, maxY: 120, centerX: 150, centerY: 100 });
    });
});

describe('systemsInBox', () => {
    const positions: [number, { x: number; y: number }][] = [
        [1, { x: 100, y: 100 }],
        [2, { x: 200, y: 200 }],
        [3, { x: 500, y: 500 }],
    ];

    it('selects anchors inside the box regardless of drag direction', () => {
        expect(systemsInBox(positions, { start: { x: 50, y: 50 }, end: { x: 250, y: 250 } })).toEqual([1, 2]);
        expect(systemsInBox(positions, { start: { x: 250, y: 250 }, end: { x: 50, y: 50 } })).toEqual([1, 2]);
    });

    it('treats the box edges as inclusive', () => {
        expect(systemsInBox(positions, { start: { x: 100, y: 100 }, end: { x: 100, y: 100 } })).toEqual([1]);
    });
});

describe('findFreePosition', () => {
    const options = { maxSize: MAX_SIZE, gridSize: GRID };

    it('starts at the padding corner on an empty map', () => {
        expect(findFreePosition([], options)).toEqual({ x: 100, y: 100 });
    });

    it('skips cells whose clearance box contains an occupied anchor', () => {
        // (100,100) blocks candidates until y stays 30 clear below it.
        expect(findFreePosition([{ x: 100, y: 100 }], options)).toEqual({ x: 100, y: 140 });
    });

    it('throws when nothing is free', () => {
        expect(() => findFreePosition([{ x: 150, y: 130 }], { maxSize: { x: 300, y: 300 }, gridSize: 100 })).toThrowError(
            'No free position found on the map',
        );
    });
});

describe('organizeIntoColumn', () => {
    it('stacks systems at the top-left of their spread, one item height plus a grid cell apart', () => {
        const current = new Map([
            [1, { x: 300, y: 400 }],
            [2, { x: 200, y: 600 }],
        ]);
        const positions = organizeIntoColumn([1, 2], current, { maxSize: MAX_SIZE, gridSize: GRID });

        // Top-left corner of the spread is (200, 400); step = 1*20 + 40 = 60.
        expect(positions.get(1)).toEqual({ x: 200, y: 400 });
        expect(positions.get(2)).toEqual({ x: 200, y: 460 });
    });

    it('clamps the column to the canvas', () => {
        const current = new Map([[1, { x: 0, y: 0 }]]);
        expect(organizeIntoColumn([1], current, { maxSize: MAX_SIZE, gridSize: GRID }).get(1)).toEqual({
            x: ANCHOR_OFFSET.x,
            y: ANCHOR_OFFSET.y,
        });
    });
});
