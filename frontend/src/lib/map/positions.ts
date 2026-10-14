// Keeping the canvas's position lookup stable between frames. A node reads its own entry,
// and an edge both of its ends, so an entry that keeps its identity while its coordinates
// do lets everything downstream skip the work for the nodes that did not move.

import type { MapConnection } from '$lib/api/types/MapConnection';
import type { MapSystemView } from '$lib/api/types/MapSystemView';
import { systemName } from './system';
import type { Vec2 } from './geometry';

/**
 * Everything the tree layout reads off the chain, as one comparable string: which nodes
 * there are, how they connect, which are pinned or home, and the names siblings sort by.
 * A status, mass or notes change leaves it alone, so the layout does not rerun for one.
 */
export function treeShapeKey(systems: MapSystemView[], connections: MapConnection[]): string {
	return JSON.stringify([
		systems.map((s) => [s.id, s.is_pinned, s.is_home, s.alias, systemName(s)]),
		connections.map((c) => [c.from_system, c.to_system]),
	]);
}

/**
 * `next` with every entry whose coordinates match `prev` swapped for the old object. When
 * nothing moved at all, `prev` itself comes back, so a reader comparing by identity sees no
 * change.
 */
export function reusePositions(
	next: Map<number, Vec2>,
	prev: ReadonlyMap<number, Vec2> | null,
): Map<number, Vec2> {
	if (!prev) return next;
	let same = next.size === prev.size;
	for (const [id, p] of next) {
		const old = prev.get(id);
		if (old && old.x === p.x && old.y === p.y) next.set(id, old);
		else same = false;
	}
	return same ? (prev as Map<number, Vec2>) : next;
}

/**
 * Where each node is drawn. An automatic layout owns every position when there is one;
 * otherwise a live drag wins, then a dropped move still waiting on the server, then the
 * stored position.
 */
export function nodePositions(
	systems: readonly MapSystemView[],
	layout: ReadonlyMap<number, Vec2> | null,
	dragged: ReadonlyMap<number, Vec2>,
	pending: Readonly<Record<number, Vec2>>,
): Map<number, Vec2> {
	if (layout) return new Map(layout);
	const out = new Map<number, Vec2>();
	for (const s of systems) {
		out.set(s.id, dragged.get(s.id) ?? pending[s.id] ?? { x: s.position_x, y: s.position_y });
	}
	return out;
}

/** How far a stored position may sit from a dropped one and still count as it landing. */
const LANDED_TOLERANCE = 0.5;

/**
 * The moves still waiting on the server once a graph payload arrived, or null when none of
 * them settled. A move settles when the server has it (ours landed) or the system is gone;
 * one the server does not have yet keeps the node where it was dropped.
 */
export function settlePending(
	pending: Readonly<Record<number, Vec2>>,
	systems: readonly MapSystemView[],
): Record<number, Vec2> | null {
	const left = { ...pending };
	let changed = false;
	for (const [key, p] of Object.entries(pending)) {
		const id = Number(key);
		const s = systems.find((s) => s.id === id);
		const landed =
			s !== undefined &&
			Math.abs(s.position_x - p.x) <= LANDED_TOLERANCE &&
			Math.abs(s.position_y - p.y) <= LANDED_TOLERANCE;
		if (!s || landed) {
			delete left[id];
			changed = true;
		}
	}
	return changed ? left : null;
}

/**
 * Which connections exist and what they join: all the edge router reads off them. Mass,
 * lifetime and size changes leave it alone.
 */
export function edgeTopologyKey(connections: MapConnection[]): string {
	return connections.map((c) => `${c.id}:${c.from_system}:${c.to_system}`).join(',');
}
