import { describe, expect, it } from 'vitest';

import { compareForTree, computeTreeLayout, type TreeInput } from './tree';
import type { MapSystemView } from '$lib/api/types/MapSystemView';

// Defaults: gridSize 20, levelGap 320, siblingGap 60, marginX 20, marginY 20.

function layout(input: Partial<TreeInput>) {
	return computeTreeLayout({ nodeIds: [], edges: [], rootIds: [], ...input });
}

describe('computeTreeLayout', () => {
	it('returns nothing for an empty map', () => {
		expect(layout({}).size).toBe(0);
	});

	it('lays a chain out left to right, one column per depth', () => {
		const positions = layout({
			nodeIds: [1, 2, 3],
			edges: [
				{ from: 1, to: 2 },
				{ from: 2, to: 3 },
			],
			rootIds: [1],
		});

		expect(positions.get(1)).toEqual({ x: 20, y: 20 });
		expect(positions.get(2)).toEqual({ x: 340, y: 20 });
		expect(positions.get(3)).toEqual({ x: 660, y: 20 });
	});

	it('roots at the home system when nothing is pinned', () => {
		const positions = layout({
			nodeIds: [1, 2],
			edges: [{ from: 1, to: 2 }],
			rootIds: [],
			homeId: 2,
		});

		expect(positions.get(2)!.x).toBe(20);
		expect(positions.get(1)!.x).toBe(340);
	});

	it('keeps home in the first column even when other systems are pinned', () => {
		// Home is three hops down the chain from the pinned system; it still roots its own
		// tree rather than hanging off that one.
		const positions = layout({
			nodeIds: [1, 2, 3, 9],
			edges: [
				{ from: 1, to: 2 },
				{ from: 2, to: 3 },
				{ from: 3, to: 9 },
			],
			rootIds: [1],
			homeId: 9,
		});

		expect(positions.get(9)!.x).toBe(20);
		expect(positions.get(1)!.x).toBe(20);
	});

	it('puts home above the pinned systems, whatever they are called', () => {
		const positions = layout({
			nodeIds: [1, 2, 9],
			edges: [
				{ from: 1, to: 2 },
				{ from: 2, to: 9 },
			],
			rootIds: [1],
			homeId: 9,
			// Sorts the pinned system first, which must not lift it above home.
			compareNodes: (a, b) => a - b,
		});

		expect(positions.get(9)!.y).toBeLessThan(positions.get(1)!.y);
	});

	it('does not root the home system twice when it is also pinned', () => {
		const positions = layout({
			nodeIds: [1, 2],
			edges: [{ from: 1, to: 2 }],
			rootIds: [1],
			homeId: 1,
		});

		expect(positions.size).toBe(2);
		expect(positions.get(1)!.x).toBe(20);
		expect(positions.get(2)!.x).toBe(340);
	});

	it('fans children out around their parent, a sibling gap apart', () => {
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
		expect(first.x).toBe(340);
		expect(second.x).toBe(340);
		expect(second.y - first.y).toBe(60);
		// Exactly between them, not a snap away from it. A sibling pitch of an even number
		// of cells is what buys this: at an odd pitch the midpoint is half a cell off the
		// grid and snapping drops the parent onto one of the two children's rows.
		expect(parent.y).toBe((first.y + second.y) / 2);
	});

	// Four children span three pitches, so the midpoint is one and a half pitches from
	// either end: the case an odd pitch cannot place on the grid at all.
	it('centres a parent on an even number of children', () => {
		const positions = layout({
			nodeIds: [1, 2, 3, 4, 5],
			edges: [
				{ from: 1, to: 2 },
				{ from: 1, to: 3 },
				{ from: 1, to: 4 },
				{ from: 1, to: 5 },
			],
			rootIds: [1],
		});
		const ys = [2, 3, 4, 5].map((id) => positions.get(id)!.y).sort((a, b) => a - b);
		expect(positions.get(1)!.y).toBe((ys[0] + ys[ys.length - 1]) / 2);
	});

	it('keeps a tall branch clear of the next one, not just its first row', () => {
		// Two branches off one root; the first has grandchildren, so the second has to
		// clear the whole subtree rather than sitting one gap below its head.
		const positions = layout({
			nodeIds: [1, 2, 3, 4, 5],
			edges: [
				{ from: 1, to: 2 },
				{ from: 2, to: 4 },
				{ from: 2, to: 5 },
				{ from: 1, to: 3 },
			],
			rootIds: [1],
		});

		const deepest = Math.max(positions.get(4)!.y, positions.get(5)!.y);
		expect(positions.get(3)!.y).toBeGreaterThanOrEqual(deepest);
	});

	// Columns land on whole cells; rows on half cells, which is what lets neighbours sit one
	// cell apart and still have their parent exactly between them.
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
			expect(y % 10).toBe(0);
		}
	});

	it('orders siblings by the comparator it is given', () => {
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

	it('keeps the pinned systems at the top of the first column', () => {
		// The stranded system sorts first by the comparator, and still goes below: the
		// column reads as the systems you pinned, then everything nothing reaches.
		const positions = layout({
			nodeIds: [1, 2, 9],
			edges: [{ from: 1, to: 2 }],
			rootIds: [1],
			compareNodes: (a, b) => (a === 9 ? -1 : b === 9 ? 1 : a - b),
		});

		expect(positions.get(1)!.x).toBe(20);
		expect(positions.get(9)!.x).toBe(20);
		expect(positions.get(1)!.y).toBeLessThan(positions.get(9)!.y);
	});

	it('sinks the systems nothing connects to below the whole chain', () => {
		const positions = layout({
			nodeIds: [1, 2, 3, 9],
			edges: [
				{ from: 1, to: 2 },
				{ from: 1, to: 3 },
			],
			rootIds: [1],
		});

		const chain = [1, 2, 3].map((id) => positions.get(id)!.y);
		expect(positions.get(9)!.y).toBeGreaterThan(Math.max(...chain));
	});

	it('parks a system nothing connects to as its own tree', () => {
		const positions = layout({
			nodeIds: [1, 2, 10],
			edges: [{ from: 1, to: 2 }],
			rootIds: [1],
		});

		expect(positions.get(10)!.x).toBe(20);
		expect(positions.get(10)!.y).not.toBe(positions.get(1)!.y);
	});

	it('ignores self-loops and edges to systems that are not on the map', () => {
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
		expect(positions.get(2)!.x).toBe(340);
	});

	it('attaches each system to the pinned root nearest it', () => {
		const positions = layout({
			nodeIds: [1, 2, 3, 4],
			edges: [
				{ from: 1, to: 3 },
				{ from: 2, to: 4 },
				{ from: 3, to: 4 },
			],
			rootIds: [1, 2],
		});

		// Both roots in the first column; the 3-4 edge must not drag either deeper.
		expect(positions.get(1)!.x).toBe(20);
		expect(positions.get(2)!.x).toBe(20);
		expect(positions.get(3)!.x).toBe(340);
		expect(positions.get(4)!.x).toBe(340);
	});
});

describe('compareForTree', () => {
	function sys(
		id: number,
		name: string,
		wormholeClass: number | null,
		security = -1,
		alias: string | null = null,
		region = 'A-R00001',
	): MapSystemView {
		return {
			kind: 'system',
			id,
			name,
			alias,
			region,
			wormhole_class_id: wormholeClass,
			security_status: security,
		} as MapSystemView;
	}

	it('puts high, low and null sec first, then wormhole space', () => {
		const systems = [
			sys(1, 'Jita', null, 0.9),
			sys(2, 'Amamake', null, 0.4),
			sys(3, 'J100001', 5),
			sys(4, 'HED-GP', null, -0.3),
			sys(5, 'J100002', 2),
			sys(6, 'Uedama', null, 0.5),
		];
		const byId = new Map(systems.map((s) => [s.id, s]));
		const names = systems
			.map((s) => s.id)
			.sort(compareForTree(byId))
			.map((id) => (byId.get(id) as { name: string }).name);

		expect(names).toEqual(['Jita', 'Uedama', 'Amamake', 'HED-GP', 'J100002', 'J100001']);
	});

	it('puts aliased systems below the rest, ordered by alias', () => {
		const systems = [
			sys(1, 'Jita', null, 0.9, 'Trade'),
			sys(2, 'J100001', 5, -1, 'Home'),
			sys(3, 'J100002', 2),
			sys(4, 'Amamake', null, 0.4),
		];
		const byId = new Map(systems.map((s) => [s.id, s]));
		const ids = systems.map((s) => s.id).sort(compareForTree(byId));

		expect(ids).toEqual([4, 3, 2, 1]);
	});

	it('breaks a class tie by region before name', () => {
		const systems = [
			sys(1, 'J100001', 5, -1, null, 'B-R00002'),
			sys(2, 'J100002', 5, -1, null, 'A-R00001'),
		];
		const byId = new Map(systems.map((s) => [s.id, s]));
		const ids = systems.map((s) => s.id).sort(compareForTree(byId));

		expect(ids).toEqual([2, 1]);
	});
});
