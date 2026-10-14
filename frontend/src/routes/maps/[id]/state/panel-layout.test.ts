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
