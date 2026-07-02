import { nodeRect } from '@/map/core/coords';
import { freeEdgeGeometry, RAIL_PADDING, railEndpoint } from '@/map/core/geometry/freeRouting';
import { describe, expect, it } from 'vitest';

// A default node: anchor (100, 100), 180×40 → box x [60, 240], y [80, 120].
const box = nodeRect({ x: 100, y: 100 }, { width: 180, height: 40 });

describe('railEndpoint', () => {
    it('sits on the horizontal centreline of the node', () => {
        expect(railEndpoint(box, 0).y).toBe(box.centerY);
    });

    it('is pulled toward the far node but clamped RAIL_PADDING inside each edge', () => {
        expect(railEndpoint(box, -1000).x).toBe(box.minX + RAIL_PADDING);
        expect(railEndpoint(box, 1000).x).toBe(box.maxX - RAIL_PADDING);
        expect(railEndpoint(box, box.centerX).x).toBe(box.centerX);
    });

    it('degrades to the centre when the box is narrower than twice the padding', () => {
        const narrow = nodeRect({ x: 100, y: 100 }, { width: 60, height: 40 });
        expect(railEndpoint(narrow, 1000).x).toBe(narrow.minX + (narrow.maxX - narrow.minX) / 2);
    });
});

describe('freeEdgeGeometry', () => {
    const other = nodeRect({ x: 800, y: 100 }, { width: 180, height: 40 });

    it('routes between the rails of two measured nodes', () => {
        const geometry = freeEdgeGeometry(1, box, other);
        expect(geometry.kind).toBe('curve');
        // Pulled toward each other: source endpoint at its right rail end, target at its left.
        expect(geometry.from).toEqual({ x: box.maxX - RAIL_PADDING, y: box.centerY });
        expect(geometry.to).toEqual({ x: other.minX + RAIL_PADDING, y: other.centerY });
    });

    it('falls back to centres/anchors while a node is unmeasured', () => {
        const geometry = freeEdgeGeometry(1, box, { x: 800, y: 100 });
        expect(geometry.from).toEqual({ x: box.centerX, y: box.centerY });
        expect(geometry.to).toEqual({ x: 800, y: 100 });
    });
});
