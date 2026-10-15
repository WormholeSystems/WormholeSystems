// The map page's tiles and the arrangements they start from. The canvas is a tile like any
// other, so canvas space can be traded against panel space.
//
// Minimum sizes live here rather than in the stored layout: they are a property of the
// panel, so tightening one reaches people who already saved a layout.

import * as v from 'valibot';

import { type GridItem, bottom, collides, compact } from '$lib/layout/grid';

export type PanelId =
	| 'map'
	| 'navigation'
	| 'system-info'
	| 'threat'
	| 'signatures'
	| 'notes'
	| 'characters'
	| 'skyhooks'
	| 'killmails'
	| 'evescout';

export interface PanelMeta {
	id: PanelId;
	label: string;
	/** Shown in the card library, so it says what you get back when you add it. */
	description: string;
	minW: number;
	minH: number;
	/** The map cannot be hidden; there would be nothing left to look at. */
	removable: boolean;
}

export const PANELS: PanelMeta[] = [
	{
		id: 'map',
		label: 'Map',
		description: 'The chain itself.',
		minW: 1,
		minH: 3,
		removable: false,
	},
	{
		id: 'navigation',
		label: 'Navigation',
		description: 'Route planner, watchlist and the Find tools.',
		minW: 1,
		minH: 2,
		removable: true,
	},
	{
		id: 'system-info',
		label: 'System',
		description: 'Class, effect, statics and external links for the active system.',
		minW: 1,
		minH: 2,
		removable: true,
	},
	{
		id: 'threat',
		label: 'Threat Analysis',
		description: 'Recent kill activity around the active wormhole system.',
		minW: 1,
		minH: 2,
		removable: true,
	},
	{
		id: 'signatures',
		label: 'Signatures',
		description: 'Scanned signatures for the active system, with paste import.',
		minW: 1,
		minH: 2,
		removable: true,
	},
	{
		id: 'notes',
		label: 'Notes',
		description: 'Free-text intel on the active system.',
		minW: 1,
		minH: 2,
		removable: true,
	},
	{
		id: 'characters',
		label: 'Pilots',
		description: 'Everyone sharing their location on this map, and how far away they are.',
		minW: 1,
		minH: 1,
		removable: true,
	},
	{
		id: 'skyhooks',
		label: 'Raidable Skyhooks',
		description: 'Skyhook theft windows across New Eden, nearest first.',
		minW: 1,
		minH: 2,
		removable: true,
	},
	{
		id: 'killmails',
		label: 'Killmails',
		description: 'What has died in the mapped systems in the last week.',
		minW: 1,
		minH: 2,
		removable: true,
	},
	{
		id: 'evescout',
		label: 'EVE Scout',
		description: 'Public wormholes out of Thera and Turnur, by how far away they are.',
		minW: 1,
		minH: 2,
		removable: true,
	},
];

export const PANEL_IDS = PANELS.map((p) => p.id);

export function panelMeta(id: PanelId): PanelMeta {
	return PANELS.find((p) => p.id === id)!;
}

export function isPanelId(v: string): v is PanelId {
	return (PANEL_IDS as string[]).includes(v);
}

export interface BreakpointMeta {
	key: BreakpointKey;
	label: string;
	/** The narrowest window this arrangement applies to. */
	minWidth: number;
	/** Added by the viewer rather than built in, so it can be removed again. */
	custom: boolean;
}

/** The four built-in keys, or one the viewer added for a wider screen. */
export type BreakpointKey = string;

export const BREAKPOINTS: BreakpointMeta[] = [
	{ key: 'xs', label: 'Phone', minWidth: 0, custom: false },
	{ key: 'sm', label: 'Tablet', minWidth: 640, custom: false },
	{ key: 'md', label: 'Laptop', minWidth: 1024, custom: false },
	{ key: 'lg', label: 'Desktop', minWidth: 1536, custom: false },
];

const BUILT_IN = new Set(BREAKPOINTS.map((bp) => bp.key));

/** A new screen size starts with a column per this many pixels, up to 12, like legacy. */
const PIXELS_PER_COLUMN = 160;
/** The server's limits on a grid. */
export const MAX_COLUMNS = 24;
export const ROW_HEIGHT_RANGE = { min: 40, max: 500 } as const;
export const MAX_BREAKPOINTS = 12;

/** Where a new screen size's column count starts; the slider can take it anywhere after. */
export function startingColumns(minWidth: number): number {
	return Math.min(12, Math.max(1, Math.floor(minWidth / PIXELS_PER_COLUMN)));
}

export interface BreakpointLayout {
	cols: number;
	row_height: number;
	items: GridItem[];
	/** Only on an added breakpoint; the built-in ones keep their widths above. */
	min_width?: number;
	label?: string;
}

export type PanelLayouts = Record<string, BreakpointLayout>;

/** The stored/pasted layout shape, for anything that arrives as untrusted JSON. */
export const panelLayoutsSchema = v.record(
	v.string(),
	v.object({
		cols: v.number(),
		row_height: v.number(),
		items: v.array(
			v.object({ i: v.string(), x: v.number(), y: v.number(), w: v.number(), h: v.number() }),
		),
		min_width: v.optional(v.number()),
		label: v.optional(v.string()),
	}),
);

/** What the layout toolbar's copy button writes, and its paste button will accept. */
export const layoutClipboardSchema = v.object({
	breakpoints: panelLayoutsSchema,
	hidden: v.optional(v.array(v.string())),
});

const item = (i: PanelId, x: number, y: number, w: number, h: number): GridItem => ({
	i,
	x,
	y,
	w,
	h,
});

/** The arrangement a map starts from, per built-in breakpoint. */
export const DEFAULT_LAYOUTS: PanelLayouts = {
	xs: {
		cols: 1,
		row_height: 100,
		items: [
			item('map', 0, 0, 1, 7),
			item('system-info', 0, 7, 1, 3),
			item('signatures', 0, 10, 1, 4),
			item('navigation', 0, 14, 1, 4),
			item('threat', 0, 18, 1, 3),
			item('notes', 0, 21, 1, 2),
			item('characters', 0, 23, 1, 3),
			item('skyhooks', 0, 26, 1, 3),
			item('killmails', 0, 29, 1, 3),
			item('evescout', 0, 32, 1, 3),
		],
	},
	sm: {
		cols: 2,
		row_height: 100,
		items: [
			item('map', 0, 0, 2, 7),
			item('system-info', 0, 7, 1, 3),
			item('signatures', 1, 7, 1, 4),
			item('navigation', 0, 10, 1, 4),
			item('threat', 1, 11, 1, 3),
			item('notes', 0, 14, 2, 2),
			item('characters', 0, 16, 2, 3),
			item('skyhooks', 0, 19, 2, 3),
			item('killmails', 0, 22, 2, 3),
			item('evescout', 0, 25, 2, 3),
		],
	},
	md: {
		cols: 4,
		row_height: 100,
		items: [
			item('map', 0, 0, 4, 8),
			item('system-info', 0, 8, 2, 3),
			item('signatures', 2, 8, 2, 4),
			item('navigation', 0, 11, 2, 4),
			item('threat', 2, 12, 2, 3),
			item('notes', 0, 15, 2, 2),
			item('characters', 2, 15, 2, 3),
			item('skyhooks', 0, 17, 2, 3),
			item('killmails', 2, 18, 2, 3),
			item('evescout', 0, 21, 2, 3),
		],
	},
	lg: {
		cols: 10,
		row_height: 100,
		items: [
			item('map', 0, 0, 7, 9),
			item('navigation', 7, 0, 3, 5),
			item('signatures', 7, 5, 3, 4),
			item('system-info', 0, 9, 3, 3),
			item('threat', 3, 9, 4, 3),
			item('notes', 7, 9, 3, 3),
			item('characters', 0, 12, 5, 3),
			item('skyhooks', 5, 12, 5, 3),
			item('killmails', 0, 15, 5, 3),
			item('evescout', 5, 15, 5, 3),
		],
	},
};

/**
 * Every breakpoint a layout has, built-in and added, narrowest first. A built-in one can
 * be renamed and moved, except that the phone size always starts at 0 so something
 * applies to every window.
 */
export function breakpointsOf(layouts: PanelLayouts | null): BreakpointMeta[] {
	const all: BreakpointMeta[] = BREAKPOINTS.map((bp) => {
		const saved = layouts?.[bp.key];
		return {
			...bp,
			label: saved?.label ?? bp.label,
			minWidth: bp.key === 'xs' ? 0 : (saved?.min_width ?? bp.minWidth),
		};
	});
	for (const [key, layout] of Object.entries(layouts ?? {})) {
		if (BUILT_IN.has(key) || layout.min_width === undefined) continue;
		all.push({ key, label: layout.label ?? key, minWidth: layout.min_width, custom: true });
	}
	return all.sort((a, b) => a.minWidth - b.minWidth || Number(a.custom) - Number(b.custom));
}

/** The breakpoint that applies at a given window width. */
export function breakpointFor(width: number, layouts: PanelLayouts | null = null): BreakpointKey {
	let match: BreakpointKey = BREAKPOINTS[0].key;
	for (const bp of breakpointsOf(layouts)) if (width >= bp.minWidth) match = bp.key;
	return match;
}

/** Panels a saved arrangement does not mention yet, appended at the bottom. */
function withMissing(items: GridItem[], fallback: GridItem[], cols: number): GridItem[] {
	const present = new Set(items.map((i) => i.i));
	let y = bottom(items);
	for (const add of fallback.filter((i) => !present.has(i.i))) {
		const w = Math.min(add.w, cols);
		items.push({ ...add, x: 0, y, w });
		y += add.h;
	}
	return compact(items, cols);
}

/**
 * The stored layout merged over the defaults. A saved arrangement that predates a panel
 * gets it appended at the bottom, so a new panel appears for everyone instead of vanishing
 * for anyone who has ever saved. Added breakpoints come through as they were saved.
 */
export function resolveLayouts(stored: PanelLayouts | null): PanelLayouts {
	const out: PanelLayouts = {};
	for (const bp of BREAKPOINTS) {
		const fallback = DEFAULT_LAYOUTS[bp.key];
		const saved = stored?.[bp.key];
		if (!saved) {
			out[bp.key] = structuredClone(fallback);
			continue;
		}
		const cols = saved.cols || fallback.cols;
		const items = saved.items.filter((i) => isPanelId(i.i));
		out[bp.key] = {
			cols,
			row_height: saved.row_height || fallback.row_height,
			items: withMissing(items, fallback.items, cols),
			...(saved.min_width !== undefined && bp.key !== 'xs' && { min_width: saved.min_width }),
			...(saved.label !== undefined && { label: saved.label }),
		};
	}
	for (const [key, saved] of Object.entries(stored ?? {})) {
		if (BUILT_IN.has(key) || saved.min_width === undefined) continue;
		const cols = saved.cols || DEFAULT_LAYOUTS.lg.cols;
		const fallback = fitToColumns(DEFAULT_LAYOUTS.lg.items, DEFAULT_LAYOUTS.lg.cols, cols);
		out[key] = {
			cols,
			row_height: saved.row_height || DEFAULT_LAYOUTS.lg.row_height,
			items: withMissing(
				saved.items.filter((i) => isPanelId(i.i)),
				fallback,
				cols,
			),
			min_width: saved.min_width,
			label: saved.label ?? key,
		};
	}
	return out;
}

/**
 * The same arrangement on a grid with a different column count: every tile keeps its
 * share of the width, and anything the rounding makes overlap moves down.
 */
export function fitToColumns(items: GridItem[], from: number, to: number): GridItem[] {
	const scale = to / from;
	const settled: GridItem[] = [];
	const ordered = [...items].sort((a, b) => a.y - b.y || a.x - b.x);
	for (const item of ordered) {
		const w = Math.min(to, Math.max(1, Math.round(item.w * scale)));
		const x = Math.min(to - w, Math.round(item.x * scale));
		const placed = { ...item, x, w };
		while (settled.some((other) => collides(placed, other))) placed.y += 1;
		settled.push(placed);
	}
	return compact(
		items.map((item) => settled.find((s) => s.i === item.i)!),
		to,
	);
}

/** A key for a new breakpoint that does not clash with one the layout already has. */
export function breakpointKey(label: string, layouts: PanelLayouts): BreakpointKey {
	const base =
		label
			.toLowerCase()
			.replace(/[^a-z0-9]+/g, '-')
			.replace(/^-+|-+$/g, '')
			.slice(0, 24) || 'screen';
	let key = base;
	for (let n = 2; key in layouts || BUILT_IN.has(key); n++) key = `${base}-${n}`;
	return key;
}

// A panel with no remembered size starts here rather than at its minimum, which can be
// a single cell.
const FRESH_W = 2;
const FRESH_H = 3;

/** Put a hidden panel back at the bottom, so unhiding never drops it into a hole. */
export function placeAtBottom(layout: BreakpointLayout, id: PanelId): BreakpointLayout {
	const meta = panelMeta(id);
	const others = layout.items.filter((i) => i.i !== id);
	const existing = layout.items.find((i) => i.i === id);
	const placed: GridItem = {
		i: id,
		x: 0,
		y: bottom(others),
		w: existing?.w ?? Math.min(Math.max(meta.minW, FRESH_W), layout.cols),
		h: existing?.h ?? Math.max(meta.minH, FRESH_H),
	};
	return { ...layout, items: compact([...others, placed], layout.cols) };
}
