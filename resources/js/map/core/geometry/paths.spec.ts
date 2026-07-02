import { CORNER_RADIUS, curvePath, edgePathAndCenter, elbowCorners, midpoint, roundedElbowPath } from '@/map/core/geometry/paths';
import type { EdgeGeometry } from '@/map/core/types';
import { describe, expect, it } from 'vitest';

describe('curvePath', () => {
    it('matches the original bezier construction', () => {
        expect(curvePath({ x: 0, y: 0 }, { x: 300, y: 90 })).toBe('M 0 0 C 200 0, 100 90, 300 90');
    });
});

describe('elbowCorners', () => {
    const base = { id: 1, kind: 'elbow', from: { x: 0, y: 0 }, to: { x: 100, y: 80 } } as const;

    it('runs the perpendicular leg at the midpoint for horizontal exits', () => {
        const corners = elbowCorners({ ...base, fromNormal: { x: 1, y: 0 }, toNormal: { x: -1, y: 0 }, bend: null });
        expect(corners).toEqual([
            { x: 50, y: 0 },
            { x: 50, y: 80 },
        ]);
    });

    it('honours an explicit bend position', () => {
        const corners = elbowCorners({ ...base, fromNormal: { x: 1, y: 0 }, toNormal: { x: -1, y: 0 }, bend: 20 });
        expect(corners[0]).toEqual({ x: 20, y: 0 });
    });

    it('bends along y for vertical exits', () => {
        const corners = elbowCorners({ ...base, fromNormal: { x: 0, y: 1 }, toNormal: { x: 0, y: -1 }, bend: null });
        expect(corners).toEqual([
            { x: 0, y: 40 },
            { x: 100, y: 40 },
        ]);
    });
});

describe('roundedElbowPath', () => {
    it('draws a straight segment with no interior corners', () => {
        expect(
            roundedElbowPath(
                [
                    { x: 0, y: 0 },
                    { x: 100, y: 0 },
                ],
                10,
            ),
        ).toBe('M 0 0 L 100 0');
    });

    it('rounds one corner with a quadratic curve', () => {
        expect(
            roundedElbowPath(
                [
                    { x: 0, y: 0 },
                    { x: 100, y: 0 },
                    { x: 100, y: 100 },
                ],
                10,
            ),
        ).toBe('M 0 0 L 90 0 Q 100 0 100 10 L 100 100');
    });

    it('clamps the radius to half the shortest adjacent segment', () => {
        expect(
            roundedElbowPath(
                [
                    { x: 0, y: 0 },
                    { x: 10, y: 0 },
                    { x: 10, y: 100 },
                ],
                10,
            ),
        ).toBe('M 0 0 L 5 0 Q 10 0 10 5 L 10 100');
    });

    it('drops duplicate points instead of emitting degenerate corners', () => {
        expect(
            roundedElbowPath(
                [
                    { x: 0, y: 0 },
                    { x: 0, y: 0 },
                    { x: 100, y: 0 },
                ],
                10,
            ),
        ).toBe('M 0 0 L 100 0');
    });

    it('returns an empty path for fewer than two distinct points', () => {
        expect(roundedElbowPath([{ x: 0, y: 0 }], 10)).toBe('');
    });
});

describe('edgePathAndCenter', () => {
    it('scales curve endpoints and centres the badge between them', () => {
        const geometry: EdgeGeometry = { id: 1, kind: 'curve', from: { x: 0, y: 0 }, to: { x: 100, y: 100 } };
        const { d, center } = edgePathAndCenter(geometry, 2);
        expect(d).toBe(curvePath({ x: 0, y: 0 }, { x: 200, y: 200 }));
        expect(center).toEqual({ x: 100, y: 100 });
    });

    it('scales elbow geometry (including the bend) but keeps the corner radius constant', () => {
        const geometry: EdgeGeometry = {
            id: 1,
            kind: 'elbow',
            from: { x: 0, y: 0 },
            to: { x: 100, y: 80 },
            fromNormal: { x: 1, y: 0 },
            toNormal: { x: -1, y: 0 },
            bend: 30,
        };
        const { d, center } = edgePathAndCenter(geometry, 2);
        // Bend 30 at scale 2 → perpendicular run at x=60; corners rounded by the unscaled radius.
        expect(d).toBe(
            roundedElbowPath(
                [
                    { x: 0, y: 0 },
                    { x: 60, y: 0 },
                    { x: 60, y: 160 },
                    { x: 200, y: 160 },
                ],
                CORNER_RADIUS,
            ),
        );
        expect(center).toEqual({ x: 60, y: 80 });
    });

    it('defaults the elbow to the scaled midpoint when bend is null', () => {
        const geometry: EdgeGeometry = {
            id: 1,
            kind: 'elbow',
            from: { x: 0, y: 0 },
            to: { x: 100, y: 0 },
            fromNormal: { x: 1, y: 0 },
            toNormal: { x: -1, y: 0 },
            bend: null,
        };
        expect(edgePathAndCenter(geometry, 2).center).toEqual({ x: 100, y: 0 });
    });
});

describe('midpoint', () => {
    it('averages the two points', () => {
        expect(midpoint({ x: 0, y: 0 }, { x: 10, y: 20 })).toEqual({ x: 5, y: 10 });
    });
});
