// Client-side route computation, ported from the legacy routing worker: Dijkstra over the
// static stargate graph plus live wormhole and EVE Scout edges. Zarzakh is always excluded.
// One single-source relaxation serves A→B routes, watchlist targets, and the closest-systems
// search alike.

import type { MassStatus } from '$lib/api/types/MassStatus';
import type { TimeStatus } from '$lib/api/types/TimeStatus';
import { ccpRoundSecurity } from '$lib/security';

export type RoutePreference = 'shorter' | 'safer' | 'less_secure';
export type RouteVia = 'stargate' | 'wormhole' | 'evescout';

export interface RouteStep {
	id: number;
	via: RouteVia | null;
}

export interface RouteResult {
	route: RouteStep[];
	jumps: number;
	cost: number;
}

export interface RoutingSettings {
	preference: RoutePreference;
	/** 0-100, weight of the security penalty in safer/less-secure modes. */
	securityPenalty: number;
	/** Worst wormhole lifetime still traversed (`stable` < `eol` < `critical`). */
	allowTimeStatus: TimeStatus;
	/** Worst wormhole mass still traversed (`stable` < `reduced` < `critical`). */
	allowMassStatus: MassStatus;
}

export interface DynamicEdge {
	a: number;
	b: number;
	via: RouteVia;
	mass: MassStatus | null;
	time: TimeStatus | null;
}

export const ZARZAKH_SYSTEM_ID = 30100000;

type Adjacency = Map<number, number[]>;
type DynamicAdjacency = Map<
	number,
	{ to: number; via: RouteVia; mass: MassStatus | null; time: TimeStatus | null }[]
>;

export interface RouteGraph {
	/** Both directions are present. */
	stargates: Adjacency;
	dynamic: DynamicAdjacency;
	/** Security by solar system id. */
	security: Map<number, number>;
}

export function buildAdjacency(edges: [number, number][]): Adjacency {
	const adj: Adjacency = new Map();
	const add = (a: number, b: number) => {
		if (a === b) return;
		const list = adj.get(a) ?? [];
		if (!list.includes(b)) list.push(b);
		adj.set(a, list);
	};
	for (const [a, b] of edges) {
		add(a, b);
		add(b, a);
	}
	return adj;
}

export function buildDynamicAdjacency(edges: DynamicEdge[]): DynamicAdjacency {
	const adj: DynamicAdjacency = new Map();
	const add = (from: number, to: number, e: DynamicEdge) => {
		if (from === to) return;
		const list = adj.get(from) ?? [];
		list.push({ to, via: e.via, mass: e.mass, time: e.time });
		adj.set(from, list);
	};
	for (const e of edges) {
		add(e.a, e.b, e);
		add(e.b, e.a, e);
	}
	return adj;
}

const TIME_RANK = { stable: 0, eol: 1, critical: 2, expired: 3 } satisfies Record<
	TimeStatus,
	number
>;
const MASS_RANK = { stable: 0, reduced: 1, critical: 2 } satisfies Record<MassStatus, number>;

/** Whether a wormhole/EVE Scout edge passes the tolerance settings (null = healthy). */
function edgeAllowed(
	settings: RoutingSettings,
	mass: MassStatus | null,
	time: TimeStatus | null,
): boolean {
	return (
		TIME_RANK[time ?? 'stable'] <= TIME_RANK[settings.allowTimeStatus] &&
		MASS_RANK[mass ?? 'stable'] <= MASS_RANK[settings.allowMassStatus]
	);
}

class PriorityQueue {
	private heap: { id: number; cost: number }[] = [];

	push(id: number, cost: number) {
		this.heap.push({ id, cost });
		let i = this.heap.length - 1;
		while (i > 0) {
			const parent = (i - 1) >> 1;
			if (this.heap[parent].cost <= this.heap[i].cost) break;
			[this.heap[parent], this.heap[i]] = [this.heap[i], this.heap[parent]];
			i = parent;
		}
	}

	pop(): { id: number; cost: number } | undefined {
		const top = this.heap[0];
		const last = this.heap.pop();
		if (this.heap.length > 0 && last) {
			this.heap[0] = last;
			let i = 0;
			for (;;) {
				const l = 2 * i + 1;
				const r = 2 * i + 2;
				let smallest = i;
				if (l < this.heap.length && this.heap[l].cost < this.heap[smallest].cost) smallest = l;
				if (r < this.heap.length && this.heap[r].cost < this.heap[smallest].cost) smallest = r;
				if (smallest === i) break;
				[this.heap[smallest], this.heap[i]] = [this.heap[i], this.heap[smallest]];
				i = smallest;
			}
		}
		return top;
	}

	get size() {
		return this.heap.length;
	}
}

/** Per-target-system edge cost, matching the legacy cost function. */
function edgeCost(settings: RoutingSettings, targetSecurity: number | undefined): number {
	if (settings.preference === 'shorter') return 1;
	const penalty = Math.exp(0.15 * settings.securityPenalty);
	const sec = ccpRoundSecurity(targetSecurity ?? 0);
	if (settings.preference === 'safer') {
		if (sec <= 0) return 2 * penalty;
		if (sec < 0.45) return penalty;
		return 0.9;
	}
	// less_secure
	if (sec <= 0) return 2 * penalty;
	if (sec < 0.45) return 0.9;
	return penalty;
}

interface Relaxation {
	dist: Map<number, number>;
	prev: Map<number, { id: number; via: RouteVia }>;
}

/**
 * The single-source relaxation behind every query. `onSettle` sees each node when its final
 * distance is known (in cost order) and may return `false` to stop early.
 */
function relax(
	graph: RouteGraph,
	from: number,
	settings: RoutingSettings,
	ignored: ReadonlySet<number>,
	endpoints: ReadonlySet<number>,
	onSettle?: (id: number) => boolean,
): Relaxation {
	const isIgnored = (id: number) =>
		!endpoints.has(id) && (id === ZARZAKH_SYSTEM_ID || ignored.has(id));

	const dist = new Map<number, number>();
	const prev = new Map<number, { id: number; via: RouteVia }>();
	const settled = new Set<number>();
	const queue = new PriorityQueue();
	dist.set(from, 0);
	queue.push(from, 0);

	while (queue.size > 0) {
		const current = queue.pop()!;
		if (current.cost > (dist.get(current.id) ?? Infinity)) continue;
		if (settled.has(current.id)) continue;
		settled.add(current.id);
		if (onSettle && !onSettle(current.id)) break;

		const step = (next: number, via: RouteVia) => {
			if (isIgnored(next)) return;
			const cost = current.cost + edgeCost(settings, graph.security.get(next));
			if (cost < (dist.get(next) ?? Infinity)) {
				dist.set(next, cost);
				prev.set(next, { id: current.id, via });
				queue.push(next, cost);
			}
		};
		for (const next of graph.stargates.get(current.id) ?? []) step(next, 'stargate');
		for (const edge of graph.dynamic.get(current.id) ?? []) {
			if (edgeAllowed(settings, edge.mass, edge.time)) step(edge.to, edge.via);
		}
	}
	return { dist, prev };
}

function reconstruct(relaxation: Relaxation, from: number, to: number): RouteResult | null {
	if (!relaxation.dist.has(to)) return null;
	if (from === to) return { route: [{ id: from, via: null }], jumps: 0, cost: 0 };
	// Each step carries the means of ARRIVING at it.
	const route: RouteStep[] = [];
	let cursor: number | undefined = to;
	while (cursor !== undefined) {
		const p = relaxation.prev.get(cursor);
		route.push({ id: cursor, via: p?.via ?? null });
		cursor = p?.id;
	}
	route.reverse();
	return { route, jumps: Math.max(0, route.length - 1), cost: relaxation.dist.get(to)! };
}

export function findRoute(
	graph: RouteGraph,
	from: number,
	to: number,
	settings: RoutingSettings,
	ignored: ReadonlySet<number> = new Set(),
): RouteResult | null {
	if (from === to) return { route: [{ id: from, via: null }], jumps: 0, cost: 0 };
	const relaxation = relax(graph, from, settings, ignored, new Set([from, to]), (id) => id !== to);
	return reconstruct(relaxation, from, to);
}

/** All targets from one origin in a single relaxation (the watchlist path). */
export function findRoutes(
	graph: RouteGraph,
	from: number,
	targets: number[],
	settings: RoutingSettings,
	ignored: ReadonlySet<number> = new Set(),
): Map<number, RouteResult> {
	const wanted = new Set(targets);
	const remaining = new Set(targets);
	const relaxation = relax(graph, from, settings, ignored, new Set([from, ...targets]), (id) => {
		remaining.delete(id);
		return remaining.size > 0;
	});
	const out = new Map<number, RouteResult>();
	for (const target of wanted) {
		const result = reconstruct(relaxation, from, target);
		if (result) out.set(target, result);
	}
	return out;
}

/** Shared so the watchlist and the pilots card agree on what "close" means. */
export function jumpTone(jumps: number): string {
	if (jumps < 8) return 'text-green-400';
	if (jumps < 15) return 'text-amber-400';
	return 'text-red-400';
}

export interface ClosestSystem {
	id: number;
	jumps: number;
	route: RouteStep[];
}

/** The nearest systems (in cost order) matching a condition (the Find path). */
export function findClosestSystems(
	graph: RouteGraph,
	from: number,
	matches: (id: number) => boolean,
	limit: number,
	settings: RoutingSettings,
	ignored: ReadonlySet<number> = new Set(),
): ClosestSystem[] {
	const hits: number[] = [];
	const relaxation = relax(graph, from, settings, ignored, new Set([from]), (id) => {
		if (matches(id)) hits.push(id);
		return hits.length < limit;
	});
	return hits.map((id) => {
		const result = reconstruct(relaxation, from, id);
		return { id, jumps: result?.jumps ?? 0, route: result?.route ?? [] };
	});
}
