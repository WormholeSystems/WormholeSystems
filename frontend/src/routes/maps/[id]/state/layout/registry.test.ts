import * as v from 'valibot';
import { describe, expect, it } from 'vitest';

import type { GridItem } from '$lib/layout/grid';
import {
	DEFAULT_LAYOUTS,
	PANEL_IDS,
	breakpointFor,
	breakpointsOf,
	fitToColumns,
	layoutClipboardSchema,
	panelMeta,
	placeAtBottom,
	resolveLayouts,
	type BreakpointLayout,
} from './registry';

describe('breakpointFor', () => {
	it('picks the widest breakpoint the window still clears', () => {
		expect(breakpointFor(0)).toBe('xs');
		expect(breakpointFor(639)).toBe('xs');
		expect(breakpointFor(640)).toBe('sm');
		expect(breakpointFor(1023)).toBe('sm');
		expect(breakpointFor(1024)).toBe('md');
		expect(breakpointFor(1535)).toBe('md');
		expect(breakpointFor(1536)).toBe('lg');
		expect(breakpointFor(5000)).toBe('lg');
	});
});

describe('resolveLayouts', () => {
	it('hands back the defaults when nothing was ever saved', () => {
		const resolved = resolveLayouts(null);
		expect(resolved).toEqual(DEFAULT_LAYOUTS);
		// A copy, not the shared default: the caller mutates its layouts.
		expect(resolved.lg).not.toBe(DEFAULT_LAYOUTS.lg);
	});

	it('fills any breakpoint the stored layout does not cover from the defaults', () => {
		const resolved = resolveLayouts({ lg: structuredClone(DEFAULT_LAYOUTS.lg) });
		expect(resolved.xs).toEqual(DEFAULT_LAYOUTS.xs);
		expect(resolved.md).toEqual(DEFAULT_LAYOUTS.md);
	});

	it('appends a panel the saved arrangement predates instead of dropping it', () => {
		const saved = structuredClone(DEFAULT_LAYOUTS.lg);
		saved.items = saved.items.filter((i) => i.i !== 'evescout');
		const resolved = resolveLayouts({ lg: saved });
		const ids = resolved.lg.items.map((i) => i.i);
		expect(ids).toEqual(expect.arrayContaining([...PANEL_IDS]));
		// It landed after everything that was already placed.
		const added = resolved.lg.items.find((i) => i.i === 'evescout')!;
		for (const other of resolved.lg.items) {
			if (other.i !== 'evescout') expect(other.y).toBeLessThanOrEqual(added.y);
		}
	});

	it('drops a stored item whose panel no longer exists', () => {
		const saved = structuredClone(DEFAULT_LAYOUTS.lg);
		saved.items.push({ i: 'retired-panel', x: 0, y: 99, w: 2, h: 2 });
		const resolved = resolveLayouts({ lg: saved });
		expect(resolved.lg.items.some((i) => i.i === 'retired-panel')).toBe(false);
	});

	it('falls back to default columns and row height when the saved ones are zero', () => {
		const saved = structuredClone(DEFAULT_LAYOUTS.lg);
		saved.cols = 0;
		saved.row_height = 0;
		const resolved = resolveLayouts({ lg: saved });
		expect(resolved.lg.cols).toBe(DEFAULT_LAYOUTS.lg.cols);
		expect(resolved.lg.row_height).toBe(DEFAULT_LAYOUTS.lg.row_height);
	});
});

describe('placeAtBottom', () => {
	const layout = (items: GridItem[]): BreakpointLayout => ({ cols: 4, row_height: 100, items });

	it('puts the panel back below everything else', () => {
		const start = layout([
			{ i: 'map', x: 0, y: 0, w: 4, h: 4 },
			{ i: 'notes', x: 0, y: 4, w: 2, h: 2 },
		]);
		const placed = placeAtBottom(start, 'signatures');
		const added = placed.items.find((i) => i.i === 'signatures')!;
		expect(added.x).toBe(0);
		for (const other of placed.items) {
			if (other.i !== 'signatures') expect(other.y).toBeLessThanOrEqual(added.y);
		}
	});

	it('keeps the size the panel had before it was hidden', () => {
		const start = layout([
			{ i: 'map', x: 0, y: 0, w: 4, h: 4 },
			{ i: 'signatures', x: 2, y: 0, w: 3, h: 5 },
		]);
		const placed = placeAtBottom(start, 'signatures');
		const added = placed.items.find((i) => i.i === 'signatures')!;
		expect(added.w).toBe(3);
		expect(added.h).toBe(5);
	});

	it('sizes a panel it has never seen at two columns by three rows', () => {
		const start = layout([{ i: 'map', x: 0, y: 0, w: 4, h: 4 }]);
		const placed = placeAtBottom(start, 'signatures');
		const added = placed.items.find((i) => i.i === 'signatures')!;
		expect(added.w).toBe(2);
		expect(added.h).toBe(3);
	});
});

describe('layoutClipboardSchema', () => {
	it('round-trips what the copy button writes', () => {
		const payload = { breakpoints: DEFAULT_LAYOUTS, hidden: ['skyhooks'] };
		const parsed = v.safeParse(layoutClipboardSchema, JSON.parse(JSON.stringify(payload)));
		expect(parsed.success).toBe(true);
	});

	it('rejects garbage and half-shaped layouts', () => {
		expect(v.safeParse(layoutClipboardSchema, 'nonsense').success).toBe(false);
		expect(v.safeParse(layoutClipboardSchema, { breakpoints: { lg: { cols: 12 } } }).success).toBe(
			false,
		);
	});
});

describe('added screen sizes', () => {
	const wide = (): BreakpointLayout => ({
		cols: 16,
		row_height: 100,
		items: [{ i: 'map', x: 0, y: 0, w: 12, h: 9 }],
		min_width: 2560,
		label: 'Ultrawide',
	});

	it('takes over from its width', () => {
		const layouts = resolveLayouts({ ultrawide: wide() });
		expect(breakpointFor(2559, layouts)).toBe('lg');
		expect(breakpointFor(2560, layouts)).toBe('ultrawide');
		expect(breakpointsOf(layouts).map((bp) => bp.key)).toEqual([
			'xs',
			'sm',
			'md',
			'lg',
			'ultrawide',
		]);
	});

	it('survives resolving, with the panels it lacks appended', () => {
		const resolved = resolveLayouts({ ultrawide: wide() }).ultrawide;
		expect(resolved).toMatchObject({ cols: 16, min_width: 2560, label: 'Ultrawide' });
		expect(resolved.items).toHaveLength(PANEL_IDS.length);
		expect(resolved.items.every((i) => i.x + i.w <= 16)).toBe(true);
	});

	it('ignores a stored key that is neither built in nor has a width', () => {
		const stray = { ...wide(), min_width: undefined };
		expect(resolveLayouts({ stray }).stray).toBeUndefined();
	});
});

describe('fitToColumns', () => {
	it('keeps each tile its share of the width without overlaps', () => {
		const items: GridItem[] = [
			{ i: 'map', x: 0, y: 0, w: 7, h: 4 },
			{ i: 'notes', x: 7, y: 0, w: 3, h: 2 },
		];
		const out = fitToColumns(items, 10, 4);
		expect(out).toEqual([
			{ i: 'map', x: 0, y: 0, w: 3, h: 4 },
			{ i: 'notes', x: 3, y: 0, w: 1, h: 2 },
		]);
	});
});

describe('a moved built-in size', () => {
	it('takes over from its new width under its new name', () => {
		const lg = { ...structuredClone(DEFAULT_LAYOUTS.lg), min_width: 1800, label: 'Big desk' };
		const layouts = resolveLayouts({ lg });
		expect(breakpointFor(1700, layouts)).toBe('md');
		expect(breakpointFor(1800, layouts)).toBe('lg');
		expect(breakpointsOf(layouts).find((bp) => bp.key === 'lg')).toMatchObject({
			label: 'Big desk',
			minWidth: 1800,
		});
	});
});
