import { describe, expect, it } from 'vitest';

import { dragMembers, bandSelection, draggedPositions, type Drag } from './gestures';
import { NODE_W } from './geometry';

const NODE_H = 40;

describe('draggedPositions', () => {
	const drag = (x: number, y: number, members: Drag['members']): Drag => ({
		primary: 1,
		x,
		y,
		offX: 0,
		offY: 0,
		members,
	});

	it('moves every member by the delta the primary moved', () => {
		const positions = draggedPositions(
			drag(130, 45, [
				{ id: 1, sx: 100, sy: 40 },
				{ id: 2, sx: 300, sy: 200 },
			]),
		);
		expect(positions.get(1)).toEqual({ x: 130, y: 45 });
		expect(positions.get(2)).toEqual({ x: 330, y: 205 });
	});

	it('is the start positions when nothing has moved', () => {
		const positions = draggedPositions(
			drag(100, 40, [
				{ id: 1, sx: 100, sy: 40 },
				{ id: 2, sx: 300, sy: 200 },
			]),
		);
		expect(positions.get(2)).toEqual({ x: 300, y: 200 });
	});

	it('applies no delta when the primary is not among the members', () => {
		const positions = draggedPositions(drag(500, 500, [{ id: 2, sx: 300, sy: 200 }]));
		expect(positions.get(2)).toEqual({ x: 300, y: 200 });
	});
});

describe('bandSelection', () => {
	const nodes = [
		{ id: 1, x: 0, y: 0 },
		{ id: 2, x: 400, y: 0 },
		{ id: 3, x: 0, y: 400 },
	];

	it('selects the nodes whose center the band covers', () => {
		const band = { x0: 0, y0: 0, x1: NODE_W, y1: NODE_H };
		expect(bandSelection(band, nodes, NODE_H)).toEqual(new Set([1]));
	});

	it('works whichever way the band was dragged', () => {
		const band = { x0: NODE_W, y0: NODE_H, x1: 0, y1: 0 };
		expect(bandSelection(band, nodes, NODE_H)).toEqual(new Set([1]));
	});

	it('misses a node the band only clips off-center', () => {
		// Covers node 1's left edge but stops short of its center.
		const band = { x0: 0, y0: 0, x1: NODE_W / 2 - 1, y1: NODE_H };
		expect(bandSelection(band, nodes, NODE_H)).toEqual(new Set());
	});

	it('takes everything when the band spans the field', () => {
		const band = { x0: -10, y0: -10, x1: 600, y1: 600 };
		expect(bandSelection(band, nodes, NODE_H)).toEqual(new Set([1, 2, 3]));
	});
});

describe('dragMembers', () => {
	const systems = [
		{ id: 1, position_x: 0, position_y: 0, is_pinned: false },
		{ id: 2, position_x: 100, position_y: 0, is_pinned: false },
		{ id: 3, position_x: 200, position_y: 0, is_pinned: true },
	];

	it('drags only the grabbed node when it is not part of a selection', () => {
		expect(dragMembers({ id: 1, at: { x: 5, y: 5 } }, new Set([2, 3]), systems, {})).toEqual([
			{ id: 1, sx: 5, sy: 5 },
		]);
	});

	it('co-drags the whole selection, pinned nodes excluded', () => {
		const members = dragMembers({ id: 1, at: { x: 0, y: 0 } }, new Set([1, 2, 3]), systems, {});
		expect(members).toEqual([
			{ id: 1, sx: 0, sy: 0 },
			{ id: 2, sx: 100, sy: 0 },
		]);
	});

	it('starts a member from its pending position over the server one', () => {
		const members = dragMembers({ id: 1, at: { x: 0, y: 0 } }, new Set([1, 2]), systems, {
			2: { x: 140, y: 60 },
		});
		expect(members).toContainEqual({ id: 2, sx: 140, sy: 60 });
	});

	it('drops a selected id that no longer exists', () => {
		const members = dragMembers({ id: 1, at: { x: 0, y: 0 } }, new Set([1, 99]), systems, {});
		expect(members).toEqual([{ id: 1, sx: 0, sy: 0 }]);
	});
});
