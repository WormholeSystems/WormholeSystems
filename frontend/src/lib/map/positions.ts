// Keeping the canvas's position lookup stable between frames. A node reads its own entry,
// and an edge both of its ends, so an entry that keeps its identity while its coordinates
// do lets everything downstream skip the work for the nodes that did not move.

import type { MapConnection } from '$lib/api/types/MapConnection';
import type { MapSystemView } from '$lib/api/types/MapSystemView';
import { systemName } from './system';
import type { Vec2 } from './helpers';

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
 * Which connections exist and what they join: all the edge router reads off them. Mass,
 * lifetime and size changes leave it alone.
 */
export function edgeTopologyKey(connections: MapConnection[]): string {
	return connections.map((c) => `${c.id}:${c.from_system}:${c.to_system}`).join(',');
}
