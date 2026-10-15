// Arrange mode for the side panels. The draft is what the grid renders, so a drag shows
// immediately; it is only persisted on Save, which is what makes Discard possible. Hidden
// panels live in the viewer's settings, reached through the host, but their dirty-tracking
// belongs here: hiding a panel is a layout change like any other.

import type { GridItem } from '$lib/layout/grid';
import type { BreakpointKey, PanelId, PanelLayouts } from './layout/registry';
import {
	DEFAULT_LAYOUTS,
	breakpointKey,
	breakpointsOf,
	fitToColumns,
	placeAtBottom,
	resolveLayouts,
} from './layout/registry';

export interface NewBreakpoint {
	label: string;
	minWidth: number;
	cols: number;
	rowHeight: number;
}

export interface LayoutHost {
	/** Null before the viewer's settings exist; hiding and showing are no-ops then. */
	hiddenPanels(): string[] | null;
	setHiddenPanels(panels: string[]): void;
	/** Persist the arrangement; resolves with the breakpoints as saved. */
	save(layouts: PanelLayouts, hidden: string[]): Promise<PanelLayouts | null>;
}

export class PanelLayoutStore {
	private host: LayoutHost;

	editing = $state(false);
	/** Raised when leaving edit mode with unsaved changes, so nothing is lost silently. */
	exitPrompt = $state(false);
	breakpoint = $state<BreakpointKey>('lg');
	draft = $state<PanelLayouts | null>(null);
	/** The last saved arrangement, for dirty-tracking and for reverting to. */
	saved = $state<PanelLayouts | null>(null);

	// A hidden-panel change never touches the draft, so it needs its own dirty flag.
	private hiddenDirty = $state(false);
	dirty = $derived(JSON.stringify(this.draft) !== JSON.stringify(this.saved) || this.hiddenDirty);

	/** Hidden panels as of the last save, so a revert restores them too. */
	private savedHidden: string[] = [];

	constructor(host: LayoutHost) {
		this.host = host;
	}

	/** Adopt a freshly loaded arrangement as both the saved state and the working copy. */
	seed(saved: PanelLayouts | null) {
		this.saved = saved;
		this.draft = structuredClone($state.snapshot(this.saved));
	}

	/**
	 * `items` only covers the visible panels; the hidden ones are carried over, since their
	 * stored positions are what put them back when they are unhidden.
	 */
	setItems(key: BreakpointKey, items: GridItem[]) {
		const base = resolveLayouts(this.draft);
		const hidden = new Set(this.host.hiddenPanels() ?? []);
		const kept = base[key].items.filter((i) => hidden.has(i.i));
		this.draft = { ...base, [key]: { ...base[key], items: [...items, ...kept] } };
	}

	set(layouts: PanelLayouts) {
		this.draft = layouts;
	}

	hidePanel(id: string) {
		const hidden = this.host.hiddenPanels();
		if (hidden === null || hidden.includes(id)) return;
		this.host.setHiddenPanels([...hidden, id]);
		this.hiddenDirty = true;
	}

	showPanel(id: string) {
		const hidden = this.host.hiddenPanels();
		if (hidden === null) return;
		// Put it back at the bottom of every breakpoint, so unhiding never drops a tile
		// into a hole left by something that has since moved.
		const base = resolveLayouts(this.draft);
		for (const key of Object.keys(base)) {
			base[key] = placeAtBottom(base[key], id as PanelId);
		}
		this.draft = base;
		this.host.setHiddenPanels(hidden.filter((p) => p !== id));
		this.hiddenDirty = true;
	}

	save() {
		const layouts = resolveLayouts(this.draft);
		this.host
			.save(layouts, this.host.hiddenPanels() ?? [])
			.then((saved) => {
				this.saved = saved;
				this.draft = structuredClone($state.snapshot(this.saved));
				this.hiddenDirty = false;
				this.editing = false;
			})
			// The host reports the failure; edit mode stays open with the draft intact.
			.catch(() => {});
	}

	/** Leave arrange mode; unsaved changes raise the prompt instead of vanishing. */
	exitEdit() {
		if (this.dirty) {
			this.exitPrompt = true;
			return;
		}
		this.editing = false;
	}

	/** Answer the prompt: keep the changes, or throw them away. */
	resolveExit(save: boolean) {
		this.exitPrompt = false;
		if (save) {
			this.save();
			return;
		}
		this.revert();
		this.editing = false;
	}

	/** Throw the working copy away and go back to what was last saved. */
	revert() {
		this.draft = structuredClone($state.snapshot(this.saved));
		this.host.setHiddenPanels(this.savedHidden);
		this.hiddenDirty = false;
	}

	rememberHidden() {
		this.savedHidden = [...(this.host.hiddenPanels() ?? [])];
	}

	/**
	 * Put one breakpoint back to the built-in arrangement. An added one has none of its
	 * own, so it gets the desktop arrangement stretched to its columns.
	 */
	reset(key: BreakpointKey) {
		const base = resolveLayouts(this.draft);
		const current = base[key];
		if (!current) return;
		if (key in DEFAULT_LAYOUTS) {
			// The arrangement goes back; a name or width the viewer gave it stays.
			const { min_width, label } = current;
			this.draft = {
				...base,
				[key]: {
					...structuredClone(DEFAULT_LAYOUTS[key]),
					...(min_width !== undefined && { min_width }),
					...(label !== undefined && { label }),
				},
			};
			return;
		}
		const from = DEFAULT_LAYOUTS.lg;
		this.draft = {
			...base,
			[key]: { ...current, items: fitToColumns(from.items, from.cols, current.cols) },
		};
	}

	/** Change one breakpoint's grid; the tiles keep their share of the width. */
	setGrid(key: BreakpointKey, grid: { cols?: number; rowHeight?: number }) {
		const base = resolveLayouts(this.draft);
		const current = base[key];
		if (!current) return;
		const cols = grid.cols ?? current.cols;
		this.draft = {
			...base,
			[key]: {
				...current,
				cols,
				row_height: grid.rowHeight ?? current.row_height,
				items:
					cols === current.cols ? current.items : fitToColumns(current.items, current.cols, cols),
			},
		};
	}

	/**
	 * Add a breakpoint for a wider (or narrower) screen. It starts from the arrangement of
	 * the breakpoint closest to its width, stretched to its columns, and becomes the one
	 * being edited.
	 */
	addBreakpoint(spec: NewBreakpoint): BreakpointKey {
		const base = resolveLayouts(this.draft);
		const closest = breakpointsOf(base).reduce((best, bp) =>
			Math.abs(bp.minWidth - spec.minWidth) < Math.abs(best.minWidth - spec.minWidth) ? bp : best,
		);
		const source = base[closest.key];
		const key = breakpointKey(spec.label, base);
		this.draft = {
			...base,
			[key]: {
				cols: spec.cols,
				row_height: spec.rowHeight,
				items: fitToColumns(source.items, source.cols, spec.cols),
				min_width: spec.minWidth,
				label: spec.label.trim(),
			},
		};
		this.breakpoint = key;
		return key;
	}

	/** Rename a breakpoint or move the width it starts at. */
	updateBreakpoint(key: BreakpointKey, patch: { label?: string; minWidth?: number }) {
		const base = resolveLayouts(this.draft);
		const current = base[key];
		if (!current) return;
		const next = { ...current };
		if (patch.label !== undefined) next.label = patch.label.trim();
		if (patch.minWidth !== undefined && key !== 'xs') next.min_width = patch.minWidth;
		this.draft = { ...base, [key]: next };
	}

	/** Drop an added breakpoint; the built-in ones always stay. */
	removeBreakpoint(key: BreakpointKey) {
		const base = resolveLayouts(this.draft);
		if (base[key]?.min_width === undefined) return;
		const rest = { ...base };
		delete rest[key];
		this.draft = rest;
		if (this.breakpoint === key) this.breakpoint = 'lg';
	}
}
