import { describe, expect, it } from 'vitest';

import { PanelLayoutStore } from './panel-layout.svelte';
import { DEFAULT_LAYOUTS, resolveLayouts, type PanelLayouts } from './layout/registry';

function harness(saved: PanelLayouts | null = null) {
	const calls: { layouts: PanelLayouts; hidden: string[] }[] = [];
	let hidden: string[] | null = [];
	let fail = false;
	const store = new PanelLayoutStore({
		hiddenPanels: () => hidden,
		setHiddenPanels: (panels) => {
			if (hidden !== null) hidden = panels;
		},
		save: (layouts, hiddenPanels) => {
			calls.push({ layouts, hidden: hiddenPanels });
			return fail ? Promise.reject(new Error('nope')) : Promise.resolve(layouts);
		},
	});
	store.seed(saved);
	return {
		store,
		calls,
		get hidden() {
			return hidden;
		},
		set hidden(v: string[] | null) {
			hidden = v;
		},
		set fail(v: boolean) {
			fail = v;
		},
	};
}

const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

/** A draft with the notes panel nudged, distinguishable from every default. */
function nudged(): PanelLayouts {
	const layouts = resolveLayouts(null);
	const item = layouts.lg.items.find((i) => i.i === 'notes')!;
	item.h += 1;
	return layouts;
}

describe('dirty tracking', () => {
	it('starts clean after a seed', () => {
		const h = harness();
		expect(h.store.dirty).toBe(false);
	});

	it('a changed draft is dirty, and setting it back is clean again', () => {
		const h = harness(resolveLayouts(null));
		h.store.set(nudged());
		expect(h.store.dirty).toBe(true);
		h.store.set(resolveLayouts(null));
		expect(h.store.dirty).toBe(false);
	});

	it('hiding a panel is dirty even though the draft is untouched', () => {
		const h = harness();
		h.store.hidePanel('notes');
		expect(h.hidden).toEqual(['notes']);
		expect(h.store.dirty).toBe(true);
	});

	it('hiding is a no-op with no settings to hold it', () => {
		const h = harness();
		h.hidden = null;
		h.store.hidePanel('notes');
		expect(h.store.dirty).toBe(false);
	});

	it('showing a panel places it back and is dirty', () => {
		const h = harness();
		h.hidden = ['notes'];
		h.store.showPanel('notes');
		expect(h.hidden).toEqual([]);
		expect(h.store.dirty).toBe(true);
		const items = resolveLayouts(h.store.draft).lg.items;
		expect(items.some((i) => i.i === 'notes')).toBe(true);
	});
});

describe('save', () => {
	it('persists the resolved draft and leaves edit mode clean', async () => {
		const h = harness();
		h.store.editing = true;
		h.store.set(nudged());
		h.store.hidePanel('notes');
		h.store.save();
		await flush();
		expect(h.calls).toHaveLength(1);
		expect(h.calls[0].hidden).toEqual(['notes']);
		expect(h.store.dirty).toBe(false);
		expect(h.store.editing).toBe(false);
		expect(h.store.saved).toEqual(h.calls[0].layouts);
	});

	it('keeps the draft and edit mode when the write fails', async () => {
		const h = harness();
		h.fail = true;
		h.store.editing = true;
		h.store.set(nudged());
		h.store.save();
		await flush();
		expect(h.store.dirty).toBe(true);
		expect(h.store.editing).toBe(true);
	});
});

describe('exit and revert', () => {
	it('exits straight away when nothing changed', () => {
		const h = harness();
		h.store.editing = true;
		h.store.exitEdit();
		expect(h.store.editing).toBe(false);
		expect(h.store.exitPrompt).toBe(false);
	});

	it('raises the prompt instead of dropping unsaved changes', () => {
		const h = harness();
		h.store.editing = true;
		h.store.set(nudged());
		h.store.exitEdit();
		expect(h.store.editing).toBe(true);
		expect(h.store.exitPrompt).toBe(true);
	});

	it('discarding restores the draft and the hidden panels', () => {
		const h = harness(resolveLayouts(null));
		h.store.editing = true;
		h.store.rememberHidden();
		h.store.set(nudged());
		h.store.hidePanel('notes');
		h.store.exitEdit();
		h.store.resolveExit(false);
		expect(h.store.editing).toBe(false);
		expect(h.store.exitPrompt).toBe(false);
		expect(h.store.dirty).toBe(false);
		expect(h.hidden).toEqual([]);
		expect(h.store.draft).toEqual(resolveLayouts(null));
	});

	it('answering save persists and exits', async () => {
		const h = harness();
		h.store.editing = true;
		h.store.set(nudged());
		h.store.exitEdit();
		h.store.resolveExit(true);
		await flush();
		expect(h.calls).toHaveLength(1);
		expect(h.store.editing).toBe(false);
	});
});

describe('reset', () => {
	it('puts one breakpoint back to the defaults and is dirty against the saved state', () => {
		const saved = nudged();
		const h = harness(saved);
		h.store.reset('lg');
		expect(resolveLayouts(h.store.draft).lg).toEqual(DEFAULT_LAYOUTS.lg);
		expect(h.store.dirty).toBe(true);
		// The other breakpoints keep the saved arrangement.
		expect(resolveLayouts(h.store.draft).md).toEqual(resolveLayouts(saved).md);
	});
});

describe('screen sizes', () => {
	const ultrawide = { label: 'Ultrawide', minWidth: 2560, cols: 16, rowHeight: 80 };

	it('adds a size that starts from the closest arrangement and becomes the one edited', () => {
		const h = harness();
		const key = h.store.addBreakpoint(ultrawide);
		const added = resolveLayouts(h.store.draft)[key];
		expect(key).toBe('ultrawide');
		expect(h.store.breakpoint).toBe('ultrawide');
		expect(added).toMatchObject({ cols: 16, row_height: 80, min_width: 2560, label: 'Ultrawide' });
		expect(added.items.map((i) => i.i).sort()).toEqual(
			DEFAULT_LAYOUTS.lg.items.map((i) => i.i).sort(),
		);
		expect(added.items.every((i) => i.x + i.w <= 16)).toBe(true);
	});

	it('changing the columns keeps every tile on the grid', () => {
		const h = harness();
		h.store.setGrid('lg', { cols: 20, rowHeight: 60 });
		const lg = resolveLayouts(h.store.draft).lg;
		expect(lg.cols).toBe(20);
		expect(lg.row_height).toBe(60);
		expect(lg.items.every((i) => i.w >= 1 && i.x + i.w <= 20)).toBe(true);
		expect(lg.items.find((i) => i.i === 'map')!.w).toBe(14);
	});

	it('removes an added size but never a built-in one', () => {
		const h = harness();
		const key = h.store.addBreakpoint(ultrawide);
		h.store.removeBreakpoint('lg');
		h.store.removeBreakpoint(key);
		const layouts = resolveLayouts(h.store.draft);
		expect(Object.keys(layouts).sort()).toEqual(['lg', 'md', 'sm', 'xs']);
		expect(h.store.breakpoint).toBe('lg');
	});
});

describe('editing a size', () => {
	it('renames and moves a built-in size, and reset keeps both', () => {
		const h = harness();
		h.store.updateBreakpoint('lg', { label: 'Big desk', minWidth: 1800 });
		h.store.reset('lg');
		const lg = resolveLayouts(h.store.draft).lg;
		expect(lg).toMatchObject({ label: 'Big desk', min_width: 1800 });
		expect(lg.items).toEqual(DEFAULT_LAYOUTS.lg.items);
	});

	it('never moves the phone size off zero', () => {
		const h = harness();
		h.store.updateBreakpoint('xs', { minWidth: 300 });
		expect(resolveLayouts(h.store.draft).xs.min_width).toBeUndefined();
	});
});
