import { nodeRect } from '@/map/core/coords';
import { BEND_SPACING, computeTreeEdgeGeometries, edgeCenterConnection, PARALLEL_SPACING } from '@/map/core/geometry/treeRouting';
import type { EdgeGeometry, Rect, Vec2 } from '@/map/core/types';
import { describe, expect, it } from 'vitest';

const SIZE = { width: 180, height: 40 };

function rectAt(anchor: Vec2): Rect {
    return nodeRect(anchor, SIZE);
}

function elbow(geometry: EdgeGeometry | undefined) {
    if (!geometry || geometry.kind !== 'elbow') throw new Error('expected an elbow geometry');
    return geometry;
}

describe('edgeCenterConnection', () => {
    it('uses left/right edges for horizontally separated boxes', () => {
        const source = rectAt({ x: 100, y: 100 });
        const target = rectAt({ x: 500, y: 300 });

        const ends = edgeCenterConnection(source, target);
        expect(ends.from).toEqual({ x: source.maxX, y: source.centerY });
        expect(ends.to).toEqual({ x: target.minX, y: target.centerY });
        expect(ends.fromNormal).toEqual({ x: 1, y: 0 });
        expect(ends.toNormal).toEqual({ x: -1, y: 0 });
    });

    it('mirrors for right-to-left connections', () => {
        const source = rectAt({ x: 500, y: 100 });
        const target = rectAt({ x: 100, y: 100 });

        const ends = edgeCenterConnection(source, target);
        expect(ends.from.x).toBe(source.minX);
        expect(ends.to.x).toBe(target.maxX);
        expect(ends.fromNormal).toEqual({ x: -1, y: 0 });
    });

    it('uses top/bottom edges only when the boxes share a column', () => {
        const source = rectAt({ x: 100, y: 100 });
        const target = rectAt({ x: 100, y: 400 });

        const ends = edgeCenterConnection(source, target);
        expect(ends.from).toEqual({ x: source.centerX, y: source.maxY });
        expect(ends.to).toEqual({ x: target.centerX, y: target.minY });
        expect(ends.fromNormal).toEqual({ x: 0, y: 1 });
    });
});

describe('computeTreeEdgeGeometries', () => {
    it('falls back to a curve between anchors while a node is unmeasured', () => {
        const rects = new Map([[1, rectAt({ x: 100, y: 100 })]]);
        const anchors = new Map([
            [1, { x: 100, y: 100 }],
            [2, { x: 500, y: 100 }],
        ]);

        const geometries = computeTreeEdgeGeometries([{ id: 7, sourceId: 1, targetId: 2 }], rects, anchors);
        expect(geometries.get(7)).toEqual({ id: 7, kind: 'curve', from: { x: 100, y: 100 }, to: { x: 500, y: 100 } });
    });

    it('drops edges with no measured box and no anchor', () => {
        const geometries = computeTreeEdgeGeometries([{ id: 7, sourceId: 1, targetId: 2 }], new Map(), new Map());
        expect(geometries.size).toBe(0);
    });

    it('spreads endpoints sharing a node edge PARALLEL_SPACING apart, ordered by the far end', () => {
        // One hub with two targets to its right, one above and one below.
        const rects = new Map([
            [1, rectAt({ x: 100, y: 300 })],
            [2, rectAt({ x: 500, y: 100 })],
            [3, rectAt({ x: 500, y: 500 })],
        ]);
        const geometries = computeTreeEdgeGeometries(
            [
                { id: 10, sourceId: 1, targetId: 2 },
                { id: 11, sourceId: 1, targetId: 3 },
            ],
            rects,
            new Map(),
        );

        const up = elbow(geometries.get(10));
        const down = elbow(geometries.get(11));
        const hubCenterY = rects.get(1)!.centerY;
        // The link to the upper target leaves above the centreline, the lower below.
        expect(up.from.y).toBe(hubCenterY - PARALLEL_SPACING / 2);
        expect(down.from.y).toBe(hubCenterY + PARALLEL_SPACING / 2);
        // Both still leave the hub's right edge.
        expect(up.from.x).toBe(rects.get(1)!.maxX);
        expect(down.from.x).toBe(rects.get(1)!.maxX);
    });

    it('staggers the bends of a fan so the farthest run sits closest to the node', () => {
        // A hub with three stacked targets in the next column.
        const rects = new Map([
            [1, rectAt({ x: 100, y: 300 })],
            [2, rectAt({ x: 500, y: 100 })],
            [3, rectAt({ x: 500, y: 300 })],
            [4, rectAt({ x: 500, y: 500 })],
        ]);
        const geometries = computeTreeEdgeGeometries(
            [
                { id: 10, sourceId: 1, targetId: 2 },
                { id: 11, sourceId: 1, targetId: 3 },
                { id: 12, sourceId: 1, targetId: 4 },
            ],
            rects,
            new Map(),
        );

        const bends = [elbow(geometries.get(10)).bend!, elbow(geometries.get(11)).bend!, elbow(geometries.get(12)).bend!];
        // All three bends exist, are distinct, and are BEND_SPACING apart.
        expect(new Set(bends).size).toBe(3);
        const sorted = [...bends].sort((a, b) => a - b);
        expect(sorted[1] - sorted[0]).toBeCloseTo(BEND_SPACING);
        expect(sorted[2] - sorted[1]).toBeCloseTo(BEND_SPACING);
        // The straight-across link (id 11, distance 0) bends farthest from the hub.
        expect(elbow(geometries.get(11)).bend).toBe(Math.max(...bends));
    });

    it('shrinks fan spacing to keep the runs inside the lane between columns', () => {
        // Narrow gap between the columns with many connections.
        const hub = rectAt({ x: 100, y: 300 });
        const rects = new Map<number, Rect>([[1, hub]]);
        const edges = [];
        for (let i = 0; i < 6; i++) {
            rects.set(2 + i, rectAt({ x: 320, y: 100 + i * 100 }));
            edges.push({ id: 10 + i, sourceId: 1, targetId: 2 + i });
        }
        const geometries = computeTreeEdgeGeometries(edges, rects, new Map());

        const bends = edges.map((edge) => elbow(geometries.get(edge.id)).bend!);
        const sorted = [...bends].sort((a, b) => a - b);
        const gap = sorted[1] - sorted[0];
        expect(gap).toBeLessThan(BEND_SPACING);
        // Every run stays strictly between the two column faces.
        for (const bend of bends) {
            expect(bend).toBeGreaterThan(hub.maxX);
            expect(bend).toBeLessThan(rectAt({ x: 320, y: 100 }).minX);
        }
    });
});
