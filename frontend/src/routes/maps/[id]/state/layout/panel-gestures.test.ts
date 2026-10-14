import { describe, expect, it } from 'vitest';

import type { GridItem } from '$lib/layout/grid';
import {
	beginGesture,
	floatingBox,
	keyboardLayout,
	moveGesture,
	type GridMetrics,
} from './panel-gestures';

const METRICS: GridMetrics = { cols: 12, rowHeight: 40, colWidth: 100, gridWidth: 1200 };
const item = (i: string, x: number, y: number, w = 4, h = 4): GridItem => ({ i, x, y, w, h });

// Real panels, so panelMeta answers with their actual minimums.
const ITEMS = [item('signatures', 0, 0), item('killmails', 4, 0)];

describe('moveGesture', () => {
	it('stays a tap under the hysteresis, then goes live', () => {
		const start = beginGesture('signatures', 'move', { x: 0, y: 0 }, ITEMS[0]);
		const still = moveGesture(start, { x: 3, y: 0 }, ITEMS, METRICS);
		expect(still.live).toBeNull();
		const live = moveGesture(start, { x: 5, y: 0 }, ITEMS, METRICS);
		expect(live.live).not.toBeNull();
		expect(live.dx).toBe(5);
	});

	it('snaps the live layout to the nearest cell', () => {
		const start = beginGesture('signatures', 'move', { x: 0, y: 0 }, ITEMS[0]);
		// 160px right of a 100px column rounds to two columns.
		const live = moveGesture(start, { x: 160, y: 0 }, ITEMS, METRICS);
		const moved = live.live?.find((i) => i.i === 'signatures');
		expect(moved?.x).toBe(2);
	});
});

describe('floatingBox', () => {
	it('is nothing before the drag commits, then follows the raw pixels', () => {
		const start = beginGesture('signatures', 'move', { x: 0, y: 0 }, ITEMS[0]);
		expect(floatingBox(start, METRICS)).toBeNull();
		const live = moveGesture(start, { x: 50, y: 10 }, ITEMS, METRICS);
		expect(floatingBox(live, METRICS)).toMatchObject({ left: 50, top: 10, width: 400 });
	});

	it('clamps a moved tile inside the grid on the right', () => {
		const start = beginGesture('signatures', 'move', { x: 0, y: 0 }, ITEMS[0]);
		const live = moveGesture(start, { x: 5000, y: 0 }, ITEMS, METRICS);
		expect(floatingBox(live, METRICS)?.left).toBe(METRICS.gridWidth - 400);
	});
});

describe('keyboardLayout', () => {
	it('moves with a plain arrow and resizes with shift', () => {
		// Sideways, because vertical compaction would pull a lone tile straight back up.
		const moved = keyboardLayout(ITEMS, 'killmails', 'ArrowRight', false, 12);
		expect(moved?.find((i) => i.i === 'killmails')?.x).toBe(5);
		const resized = keyboardLayout(ITEMS, 'killmails', 'ArrowRight', true, 12);
		expect(resized?.find((i) => i.i === 'killmails')?.w).toBe(5);
	});

	it('ignores keys that are not arrows', () => {
		expect(keyboardLayout(ITEMS, 'killmails', 'Enter', false, 12)).toBeNull();
	});
});
