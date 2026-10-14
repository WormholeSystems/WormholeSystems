// The map page's tiles and the arrangements they start from. The canvas is a tile like any
// other, so canvas space can be traded against panel space.
//
// Minimum sizes live here rather than in the stored layout: they are a property of the
// panel, so tightening one reaches people who already saved a layout.

import * as v from 'valibot';

import { type GridItem, bottom, compact } from '$lib/layout/grid';

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
		minW: 2,
		minH: 4,
		removable: false,
	},
	{
		id: 'navigation',
		label: 'Navigation',
		description: 'Route planner, watchlist and the Find tools.',
		minW: 2,
		minH: 3,
		removable: true,
	},
	{
		id: 'system-info',
		label: 'System',
		description: 'Class, effect, statics and external links for the active system.',
		minW: 2,
		minH: 2,
		removable: true,
	},
	{
		id: 'threat',
		label: 'Threat Analysis',
		description: 'Recent kill activity around the active wormhole system.',
		minW: 2,
		minH: 2,
		removable: true,
	},
	{
		id: 'signatures',
		label: 'Signatures',
		description: 'Scanned signatures for the active system, with paste import.',
		minW: 2,
		minH: 3,
		removable: true,
	},
	{
		id: 'notes',
		label: 'Notes',
		description: 'Free-text intel on the active system.',
		minW: 2,
		minH: 2,
		removable: true,
	},
	{
		id: 'characters',
		label: 'Pilots',
		description: 'Everyone sharing their location on this map, and how far away they are.',
		minW: 2,
		minH: 3,
		removable: true,
	},
	{
		id: 'skyhooks',
		label: 'Raidable Skyhooks',
		description: 'Skyhook theft windows across New Eden, nearest first.',
		minW: 2,
		minH: 3,
		removable: true,
	},
	{
		id: 'killmails',
		label: 'Killmails',
		description: 'What has died in the mapped systems in the last week.',
		minW: 2,
		minH: 3,
		removable: true,
	},
	{
		id: 'evescout',
		label: 'EVE Scout',
		description: 'Public wormholes out of Thera and Turnur, by how far away they are.',
		minW: 2,
		minH: 3,
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
}

export type BreakpointKey = 'xs' | 'sm' | 'md' | 'lg';

export const BREAKPOINTS: BreakpointMeta[] = [
	{ key: 'xs', label: 'Phone', minWidth: 0 },
	{ key: 'sm', label: 'Tablet', minWidth: 640 },
	{ key: 'md', label: 'Laptop', minWidth: 1024 },
	{ key: 'lg', label: 'Desktop', minWidth: 1536 },
];

export interface BreakpointLayout {
	cols: number;
	row_height: number;
	items: GridItem[];
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

/** The arrangement a map starts from, per breakpoint. */
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

/** The breakpoint that applies at a given window width. */
export function breakpointFor(width: number): BreakpointKey {
	let match: BreakpointKey = BREAKPOINTS[0].key;
	for (const bp of BREAKPOINTS) if (width >= bp.minWidth) match = bp.key;
	return match;
}

/**
 * The stored layout merged over the defaults. A saved arrangement that predates a panel
 * gets it appended at the bottom, so a new panel appears for everyone instead of vanishing
 * for anyone who has ever saved.
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
		const present = new Set(items.map((i) => i.i));
		const missing = fallback.items.filter((i) => !present.has(i.i));
		let y = bottom(items);
		for (const add of missing) {
			items.push({ ...add, x: 0, y });
			y += add.h;
		}
		out[bp.key] = {
			cols,
			row_height: saved.row_height || fallback.row_height,
			items: compact(items, cols),
		};
	}
	return out;
}

/** Put a hidden panel back at the bottom, so unhiding never drops it into a hole. */
export function placeAtBottom(layout: BreakpointLayout, id: PanelId): BreakpointLayout {
	const meta = panelMeta(id);
	const others = layout.items.filter((i) => i.i !== id);
	const existing = layout.items.find((i) => i.i === id);
	const placed: GridItem = {
		i: id,
		x: 0,
		y: bottom(others),
		w: existing?.w ?? Math.min(meta.minW, layout.cols),
		h: existing?.h ?? meta.minH,
	};
	return { ...layout, items: compact([...others, placed], layout.cols) };
}
