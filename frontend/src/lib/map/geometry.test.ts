import { describe, expect, it } from 'vitest';

import { NODE_GAP_CELLS, NODE_W, freePosition } from './geometry';
import type { GridConfig } from '$lib/api/types/GridConfig';
import type { MapSystemView } from '$lib/api/types/MapSystemView';

const grid: GridConfig = {
	cell_size: 20,
	world_width: 4000,
	world_height: 2000,
	viewport_height: 1400,
};

const GAP = NODE_GAP_CELLS * grid.cell_size;
const NODE_H = 2 * grid.cell_size;

function at(x: number, y: number): MapSystemView {
	return { position_x: x, position_y: y } as MapSystemView;
}

/** The smallest edge-to-edge distance between two nodes, per axis. */
function clearance(a: { x: number; y: number }, b: { x: number; y: number }) {
	return {
		x: Math.abs(a.x - b.x) - NODE_W,
		y: Math.abs(a.y - b.y) - NODE_H,
	};
}

describe('freePosition', () => {
	it('leaves the base alone when nothing is near it', () => {
		expect(freePosition([], { x: 300, y: 200 }, grid)).toEqual({ x: 300, y: 200 });
	});

	it('snaps to the grid', () => {
		expect(freePosition([], { x: 307, y: 193 }, grid)).toEqual({ x: 300, y: 200 });
	});

	it('steps out of a node it is anchored on, leaving the full gap', () => {
		const origin = { x: 400, y: 300 };
		const spot = freePosition([at(origin.x, origin.y)], origin, grid);

		expect(spot).toEqual({ x: origin.x + NODE_W + GAP, y: origin.y });
		// The point of the change: the two nodes no longer touch.
		expect(clearance(spot, origin).x).toBe(GAP);
	});

	it('keeps the gap from every placed node, not just the anchor', () => {
		const origin = { x: 400, y: 300 };
		const placed = [
			at(origin.x, origin.y),
			// Sitting exactly where the first free slot would otherwise be.
			at(origin.x + NODE_W + GAP, origin.y),
		];

		const spot = freePosition(placed, origin, grid);
		for (const other of placed) {
			const gaps = clearance(spot, { x: other.position_x, y: other.position_y });
			expect(gaps.x >= GAP || gaps.y >= GAP).toBe(true);
		}
	});

	it('stacks the next one under the first, instead of marching right', () => {
		const origin = { x: 400, y: 300 };
		const beside = { x: origin.x + NODE_W + GAP, y: origin.y };
		const spot = freePosition([at(origin.x, origin.y), at(beside.x, beside.y)], origin, grid);

		// Same column as the hole already mapped off this system, one row down.
		expect(spot).toEqual({ x: beside.x, y: beside.y + NODE_H + GAP });
	});

	it('moves to the next column once the column is full', () => {
		const origin = { x: 0, y: 0 };
		// Filled from the top of the world to the bottom, so there is nowhere left to drop to.
		const rows = Math.ceil(grid.world_height / (NODE_H + GAP));
		const column = Array.from({ length: rows }, (_, i) => at(NODE_W + GAP, i * (NODE_H + GAP)));
		const spot = freePosition([at(origin.x, origin.y), ...column], origin, grid);
		expect(spot.x).toBe(2 * (NODE_W + GAP));
		expect(spot.y).toBe(0);
	});

	it('stays inside the world', () => {
		const spot = freePosition([], { x: 99_999, y: 99_999 }, grid);
		expect(spot.x).toBeLessThanOrEqual(grid.world_width - NODE_W);
		expect(spot.y).toBeLessThanOrEqual(grid.world_height - NODE_H);
	});
});
