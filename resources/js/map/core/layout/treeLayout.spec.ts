import { computeTreeLayout, type TreeLayoutInput } from '@/map/core/layout/treeLayout';
import { describe, expect, it } from 'vitest';

/**
 * Characterization tests pinning the current tree layout before it moves into
 * map/core. Default options: gridSize 20, levelGap 320, siblingGap 60,
 * marginX 60, marginY 40.
 */

function layout(input: Partial<TreeLayoutInput>) {
    return computeTreeLayout({ nodeIds: [], edges: [], rootIds: [], ...input });
}

describe('computeTreeLayout', () => {
    it('returns an empty map for empty input', () => {
        expect(layout({}).size).toBe(0);
    });

    it('lays a chain out left-to-right, one level per depth', () => {
        const positions = layout({
            nodeIds: [1, 2, 3],
            edges: [
                { from: 1, to: 2 },
                { from: 2, to: 3 },
            ],
            rootIds: [1],
        });

        expect(positions.get(1)).toEqual({ x: 60, y: 40 });
        expect(positions.get(2)).toEqual({ x: 380, y: 40 });
        expect(positions.get(3)).toEqual({ x: 700, y: 40 });
    });

    it('uses the fallback root when nothing is pinned', () => {
        const positions = layout({
            nodeIds: [1, 2],
            edges: [{ from: 1, to: 2 }],
            rootIds: [],
            fallbackRootId: 2,
        });

        expect(positions.get(2)!.x).toBe(60);
        expect(positions.get(1)!.x).toBe(380);
    });

    it('fans two children out around their parent, siblingGap apart', () => {
        const positions = layout({
            nodeIds: [1, 2, 3],
            edges: [
                { from: 1, to: 2 },
                { from: 1, to: 3 },
            ],
            rootIds: [1],
        });

        const parent = positions.get(1)!;
        const first = positions.get(2)!;
        const second = positions.get(3)!;
        expect(first.x).toBe(380);
        expect(second.x).toBe(380);
        expect(second.y - first.y).toBe(60);
        // The parent sits on the snapped midpoint between its children.
        expect(Math.abs(parent.y - (first.y + second.y) / 2)).toBeLessThanOrEqual(10);
    });

    it('snaps every coordinate to the grid', () => {
        const positions = layout({
            nodeIds: [1, 2, 3, 4, 5, 6, 7],
            edges: [
                { from: 1, to: 2 },
                { from: 1, to: 3 },
                { from: 2, to: 4 },
                { from: 2, to: 5 },
                { from: 3, to: 6 },
                { from: 3, to: 7 },
            ],
            rootIds: [1],
        });

        for (const { x, y } of positions.values()) {
            expect(x % 20).toBe(0);
            expect(y % 20).toBe(0);
        }
    });

    it('orders siblings with compareNodes', () => {
        const positions = layout({
            nodeIds: [1, 2, 3],
            edges: [
                { from: 1, to: 2 },
                { from: 1, to: 3 },
            ],
            rootIds: [1],
            compareNodes: (a, b) => b - a,
        });

        expect(positions.get(3)!.y).toBeLessThan(positions.get(2)!.y);
    });

    it('parks disconnected systems as their own trees without overlaps', () => {
        const positions = layout({
            nodeIds: [1, 2, 10],
            edges: [{ from: 1, to: 2 }],
            rootIds: [1],
        });

        expect(positions.get(10)!.x).toBe(60);
        expect(positions.get(10)!.y).not.toBe(positions.get(1)!.y);
    });

    it('ignores self-loops and edges to unknown systems', () => {
        const positions = layout({
            nodeIds: [1, 2],
            edges: [
                { from: 1, to: 1 },
                { from: 1, to: 99 },
                { from: 1, to: 2 },
            ],
            rootIds: [1],
        });

        expect(positions.size).toBe(2);
        expect(positions.get(2)!.x).toBe(380);
    });

    it('attaches unpinned systems to the nearest pinned root via a shared BFS front', () => {
        const positions = layout({
            nodeIds: [1, 2, 3, 4],
            edges: [
                { from: 1, to: 3 },
                { from: 2, to: 4 },
                { from: 3, to: 4 },
            ],
            rootIds: [1, 2],
        });

        // Both roots at depth 0, their direct neighbours at depth 1 — the
        // 3-4 edge must not pull either onto a deeper level.
        expect(positions.get(1)!.x).toBe(60);
        expect(positions.get(2)!.x).toBe(60);
        expect(positions.get(3)!.x).toBe(380);
        expect(positions.get(4)!.x).toBe(380);
    });

    it('keeps every pair of nodes on the same level at least siblingGap apart', () => {
        const positions = layout({
            nodeIds: [1, 2, 3, 4, 5, 6, 7, 8, 9],
            edges: [
                { from: 1, to: 2 },
                { from: 1, to: 3 },
                { from: 2, to: 4 },
                { from: 2, to: 5 },
                { from: 2, to: 6 },
                { from: 3, to: 7 },
                { from: 3, to: 8 },
                { from: 3, to: 9 },
            ],
            rootIds: [1],
        });

        const byDepth = new Map<number, number[]>();
        for (const { x, y } of positions.values()) {
            byDepth.set(x, [...(byDepth.get(x) ?? []), y]);
        }
        for (const ys of byDepth.values()) {
            ys.sort((a, b) => a - b);
            for (let i = 1; i < ys.length; i++) {
                expect(ys[i] - ys[i - 1]).toBeGreaterThanOrEqual(60);
            }
        }
    });

    it('respects custom spacing options and snaps them to the grid', () => {
        const positions = computeTreeLayout(
            {
                nodeIds: [1, 2],
                edges: [{ from: 1, to: 2 }],
                rootIds: [1],
            },
            { levelGap: 100, marginX: 0, marginY: 0, gridSize: 40 },
        );

        expect(positions.get(1)).toEqual({ x: 0, y: 0 });
        // levelGap 100 snaps to whole grid cells: round(100 / 40) * 40 = 120.
        expect(positions.get(2)!.x).toBe(120);
    });
});
