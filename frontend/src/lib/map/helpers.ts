// Pure helpers for the map canvas.

import type { BackgroundMode } from '$lib/api/types/BackgroundMode';
import type { GridConfig } from '$lib/api/types/GridConfig';
import type { MapSystemView } from '$lib/api/types/MapSystemView';
import type { MassStatus } from '$lib/api/types/MassStatus';
import type { SystemStatus } from '$lib/api/types/SystemStatus';

/** A point in world coordinates. Named, because the map passes a great many of them. */
export interface Vec2 {
	x: number;
	y: number;
}
import type { TimeStatus } from '$lib/api/types/TimeStatus';
import type { WormholeSize } from '$lib/api/types/WormholeSize';
import { isWormholeClass } from '$lib/map/classes';
import { ccpRoundSecurity } from '$lib/security';
import type { MappedSystem } from '$lib/map/system';

/** Fixed node width (px, world space). Height is `2 * grid cell`. */
export const NODE_W = 180;

/** kg → kilotons, `en-US` grouping, one decimal, no unit suffix (legacy formatKilotons). */
export function formatKt(kg: number): string {
	return (kg / 1_000_000).toLocaleString('en-US', { maximumFractionDigits: 1 });
}

/**
 * What is left of a hole's estimated total mass after the logged jumps, clamped at empty.
 * Null when the total is unknown, so the caller can leave the bar out entirely.
 */
export function remainingMass(
	totalKg: number | null,
	jumpedKg: number,
): { kg: number; percent: number } | null {
	if (totalKg === null || totalKg <= 0) return null;
	return {
		kg: Math.max(0, totalKg - jumpedKg),
		percent: Math.max(0, 100 - (jumpedKg / totalKg) * 100),
	};
}

/** Legacy ship-size letter by max jump mass (kg). */
export function shipSizeLetter(kg: number | null): string {
	if (kg === null || kg <= 0) return '—';
	const m = kg / 1_000_000;
	if (m >= 1000) return 'XL';
	if (m >= 62) return 'L';
	if (m >= 5) return 'M';
	return 'S';
}

/** The wormhole size a hole of this jump mass admits; an identified hole dictates its own. */
export function sizeForJumpMass(kg: number | null | undefined): WormholeSize | null {
	if (!kg) return null;
	if (kg <= 5_000_000) return 'small';
	if (kg <= 300_000_000) return 'medium';
	if (kg <= 1_000_000_000) return 'large';
	return 'xl';
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

const GRID_LINES = [
	'linear-gradient(to right, var(--color-grid) 1px, transparent 1px)',
	'linear-gradient(to bottom, var(--color-grid) 1px, transparent 1px)',
];

export function gridBackground(): string {
	return GRID_LINES.join(', ');
}

interface BackgroundLayer {
	image: string;
	size: string;
	repeat: string;
	position: string;
}

function declarations(layers: BackgroundLayer[]): string {
	if (layers.length === 0) return '';
	const list = (pick: (l: BackgroundLayer) => string) => layers.map(pick).join(', ');
	return (
		`background-image: ${list((l) => l.image)}; ` +
		`background-size: ${list((l) => l.size)}; ` +
		`background-repeat: ${list((l) => l.repeat)}; ` +
		`background-position: ${list((l) => l.position)};`
	);
}

/**
 * The backgrounds of the canvas's two layers as CSS declarations: the scaled world the
 * nodes sit on, and the fixed viewport around it. A `grid` image is painted onto the world
 * so it pans and zooms with the systems; a `viewport` image fills the panel and stays put.
 */
export function canvasBackground(opts: {
	gridLines: boolean;
	cell: number;
	image: string | null;
	mode: BackgroundMode;
}): { world: string; viewport: string } {
	const cell = `${opts.cell}px ${opts.cell}px`;
	const picture: BackgroundLayer | null = opts.image
		? {
				image: `url("${opts.image.replaceAll('"', '%22')}")`,
				size: 'cover',
				repeat: 'no-repeat',
				position: 'center center',
			}
		: null;
	const world: BackgroundLayer[] = opts.gridLines
		? GRID_LINES.map((image) => ({ image, size: cell, repeat: 'repeat', position: '0 0' }))
		: [];
	if (picture && opts.mode === 'grid') world.push(picture);
	return {
		world: declarations(world),
		viewport: declarations(picture && opts.mode === 'viewport' ? [picture] : []),
	};
}

/** The node border / status icon color for a system's intel status. */
export function statusColor(status: SystemStatus): string {
	return `var(--color-status-${status})`;
}

/** Short ship-size label for a connection's max mass class. */
export function sizeLetter(s: WormholeSize): string {
	switch (s) {
		case 'xl':
			return 'XL';
		case 'large':
			return 'L';
		case 'medium':
			return 'M';
		case 'small':
			return 'S';
	}
}

/** Stroke colour by precedence: on-route, stargate, critical, reduced, EOL, else neutral. */
export function edgeColor(
	kind: 'wormhole' | 'stargate',
	mass: MassStatus | null,
	time: TimeStatus | null,
	onRoute: boolean,
): string {
	if (onRoute) return '#f59e0b'; // amber-500
	if (kind === 'stargate') return '#0ea5e9'; // sky-500
	if (mass === 'critical' || time === 'critical' || time === 'expired') return '#ef4444'; // red-500
	if (mass === 'reduced') return '#f97316'; // orange-500
	if (time === 'eol') return '#a855f7'; // purple-500
	return 'var(--color-edge)';
}

export function clamp(v: number, lo: number, hi: number): number {
	return Math.min(Math.max(v, lo), hi);
}

/**
 * A guess, not a measurement: the pair of classes rules some sizes out, and the rest is
 * left unset for whoever scans it.
 */
export function heuristicSize(
	systems: MapSystemView[],
	fromId: number,
	toId: number,
): WormholeSize | undefined {
	const a = systems.find((s) => s.id === fromId);
	const b = systems.find((s) => s.id === toId);
	// Nobody has been through an unmapped hole, so there is nothing about it to guess from.
	if (a?.kind !== 'system' || b?.kind !== 'system') return undefined;
	const TURNUR = 30002086;
	const classes = [a.wormhole_class_id, b.wormhole_class_id];
	if (classes.includes(13)) return 'small';
	if (classes.includes(1)) return 'medium';
	const highsec = (s: MappedSystem) =>
		s.wormhole_class_id === 7 || ccpRoundSecurity(s.security_status) >= 0.5;
	const thera = (s: MappedSystem) => s.wormhole_class_id === 12;
	const wh = (s: MappedSystem) => isWormholeClass(s.wormhole_class_id);
	if ((thera(a) && highsec(b)) || (thera(b) && highsec(a))) return 'medium';
	if ((a.solar_system_id === TURNUR && wh(b)) || (b.solar_system_id === TURNUR && wh(a))) {
		return 'medium';
	}
	return undefined;
}
