// Building the dynamic half of the route graph from the chain, and reading a computed
// path back onto the map's connections. Pure over the map payloads, so the rules are
// testable without a planner.

import type { EveScoutConnection } from '$lib/api/types/EveScoutConnection';
import type { MapConnection } from '$lib/api/types/MapConnection';
import type { MapSystemView } from '$lib/api/types/MapSystemView';
import type { MassStatus } from '$lib/api/types/MassStatus';
import type { TimeStatus } from '$lib/api/types/TimeStatus';
import type { DynamicEdge } from '$lib/routing/algorithm';

/** Placement id -> solar system id, for the placements that are real systems. */
function placementSystems(systems: MapSystemView[]): Map<number, number> {
	const out = new Map<number, number>();
	for (const s of systems) {
		if (s.kind === 'system') out.set(s.id, s.solar_system_id);
	}
	return out;
}

/**
 * The chain's own edges, ready for [`buildDynamicAdjacency`]. Ghosts are left out: a hole
 * whose far side is unknown leads nowhere the router could take you. Stargate-kind
 * connections (drawn to simulate Ansiblex bridges) count as plain gate jumps, so they
 * carry no mass or lifetime for the tolerance settings to drop them over. EVE Scout's
 * public holes ride along only when asked for, and so does the chain itself: a viewer who
 * has turned its holes off routes over gates alone, EVE Scout's own switch untouched.
 */
export function chainEdges(
	systems: MapSystemView[],
	connections: MapConnection[],
	eveScout: EveScoutConnection[] | null,
	useWormholes = true,
): DynamicEdge[] {
	const placement = placementSystems(systems);
	const edges: DynamicEdge[] = [];
	for (const c of connections) {
		const a = placement.get(c.from_system);
		const b = placement.get(c.to_system);
		if (a === undefined || b === undefined || a === b) continue;
		if (c.kind === 'wormhole') {
			if (!useWormholes) continue;
			edges.push({ a, b, via: 'wormhole', mass: c.mass_status, time: c.time_status });
		} else {
			edges.push({ a, b, via: 'stargate', mass: null, time: null });
		}
	}
	for (const e of eveScout ?? []) {
		edges.push({
			a: e.hub_solar_system_id,
			b: e.solar_system_id,
			via: 'evescout',
			mass: e.mass_status as MassStatus,
			time: e.time_status as TimeStatus,
		});
	}
	return edges;
}

/** Whether two [`chainEdges`] results would build the same graph. Order counts, as it does there. */
export function sameEdges(a: DynamicEdge[], b: DynamicEdge[]): boolean {
	if (a.length !== b.length) return false;
	return a.every((x, i) => {
		const y = b[i];
		return x.a === y.a && x.b === y.b && x.via === y.via && x.mass === y.mass && x.time === y.time;
	});
}

/** Connections on the active route: endpoints at adjacent path indices (legacy rule). */
export function routeConnectionIds(
	path: number[],
	systems: MapSystemView[],
	connections: MapConnection[],
): Set<number> {
	const out = new Set<number>();
	if (path.length < 2) return out;
	const index = new Map<number, number>();
	path.forEach((id, i) => index.set(id, i));
	const placement = placementSystems(systems);
	for (const c of connections) {
		const a = index.get(placement.get(c.from_system) ?? -1);
		const b = index.get(placement.get(c.to_system) ?? -1);
		if (a !== undefined && b !== undefined && Math.abs(a - b) === 1) out.add(c.id);
	}
	return out;
}
