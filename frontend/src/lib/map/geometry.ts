// World-space geometry of the map canvas: node bounds, placement and connection endpoints.

import type { GridConfig } from '$lib/api/types/GridConfig';
import type { MapSystemView } from '$lib/api/types/MapSystemView';

/** A point in world coordinates. Named, because the map passes a great many of them. */
export interface Vec2 {
	x: number;
	y: number;
}

/** Fixed node width (px, world space). Height is `2 * grid cell`. */
export const NODE_W = 180;

export function clamp(v: number, lo: number, hi: number): number {
	return Math.min(Math.max(v, lo), hi);
}

/** World coords of the viewport center (where a freshly-added system lands). */
export function centerWorld(
	pan: Vec2,
	zoom: number,
	viewport: { width: number; height: number },
): Vec2 {
	return {
		x: (viewport.width / 2 - pan.x) / zoom,
		y: (viewport.height / 2 - pan.y) / zoom,
	};
}

/** Clear space kept between placed nodes, in grid cells. */
export const NODE_GAP_CELLS = 1;

/**
 * The first free, grid-snapped slot at/after `base`: beside it, then down that column.
 * Free means [`NODE_GAP_CELLS`] of clear space, not merely non-overlapping. Holes off one
 * system are siblings, so they stack down the column and only move right when it is full.
 */
export function freePosition(
	systems: { position_x: number; position_y: number }[],
	base: Vec2,
	g: GridConfig,
): Vec2 {
	const nodeH = 2 * g.cell_size;
	const gap = NODE_GAP_CELLS * g.cell_size;
	const stepX = NODE_W + gap;
	const stepY = nodeH + gap;
	const maxX = g.world_width - NODE_W;
	const maxY = g.world_height - nodeH;
	const snap = (v: number) => Math.round(v / g.cell_size) * g.cell_size;
	const crowded = (x: number, y: number) =>
		systems.some((s) => Math.abs(x - s.position_x) < stepX && Math.abs(y - s.position_y) < stepY);
	const bx = snap(clamp(base.x, 0, maxX));
	const by = snap(clamp(base.y, 0, maxY));

	if (!crowded(bx, by)) return { x: bx, y: by };

	for (let column = 1; ; column++) {
		const x = snap(bx + column * stepX);
		if (x > maxX) break;
		for (let row = 0; ; row++) {
			const y = snap(by + row * stepY);
			if (y > maxY) break;
			if (!crowded(x, y)) return { x, y };
		}
	}
	return { x: bx, y: by };
}

/** The placement id whose node bounds contain the world point, if any. */
export function nodeAt(
	systems: MapSystemView[],
	wx: number,
	wy: number,
	g: GridConfig,
	/** Where the nodes actually are: an automatic layout overrides the stored position. */
	positions?: ReadonlyMap<number, { x: number; y: number }>,
): number | null {
	const h = 2 * g.cell_size;
	const hit = systems.find((s) => {
		const at = positions?.get(s.id) ?? { x: s.position_x, y: s.position_y };
		return wx >= at.x && wx <= at.x + NODE_W && wy >= at.y && wy <= at.y + h;
	});
	return hit ? hit.id : null;
}

/** Padding of the connection endpoint "rail" inside each vertical node edge (legacy). */
export const RAIL_PADDING = 40;

/**
 * An endpoint sliding along a horizontal rail through the node's centre, pulled toward the
 * far node but never closer than RAIL_PADDING to the edge.
 */
export function railEndpoint(minX: number, maxX: number, centerY: number, towardX: number): Vec2 {
	const padding = Math.min(RAIL_PADDING, (maxX - minX) / 2);
	return { x: clamp(towardX, minX + padding, maxX - padding), y: centerY };
}
