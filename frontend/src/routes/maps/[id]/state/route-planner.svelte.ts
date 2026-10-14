// Route planning over the chain: the static universe tables, the graph derived from them
// plus the map's own edges, and the pinned A→B route with its hover override.

import type { RoutingGraph as RoutingTables } from '$lib/api/types/RoutingGraph';
import type { EveScoutConnection } from '$lib/api/types/EveScoutConnection';
import type { MapConnection } from '$lib/api/types/MapConnection';
import type { MapSystemView } from '$lib/api/types/MapSystemView';
import type { Signature } from '$lib/api/types/Signature';
import type {
	DynamicEdge,
	RouteGraph,
	RouteSearch,
	RouteStep,
	RoutingSettings,
} from '$lib/routing/algorithm';
import { buildDynamicAdjacency } from '$lib/routing/algorithm';
import { RouteSearchCache } from '$lib/routing/search-cache';
import * as graphBuild from '$lib/routing/graph-build';
import * as routeSignatures from '$lib/routing/signatures';
import * as v from 'valibot';
import { readStored, removeStored, writeStored } from '$lib/storage';

/** Somewhere worth routing to, and the stations that make it worth it. */
export interface StationGroup {
	id: number;
	name: string;
	/** The owning corporation's faction; null for service groups. */
	faction: { id: number; name: string } | null;
	systems: Set<number>;
	/** Concrete stations per system, so results can name (and target) the station. */
	stationsBySystem: Map<number, { id: number; name: string }[]>;
}

function stationGroup(group: {
	id: number;
	name: string;
	faction?: { id: number; name: string };
	stations: { id: number; name: string; solar_system_id: number }[];
}): StationGroup {
	const stationsBySystem = new Map<number, { id: number; name: string }[]>();
	for (const station of group.stations) {
		const list = stationsBySystem.get(station.solar_system_id) ?? [];
		list.push({ id: station.id, name: station.name });
		stationsBySystem.set(station.solar_system_id, list);
	}
	return {
		id: group.id,
		name: group.name,
		faction: group.faction ?? null,
		systems: new Set(stationsBySystem.keys()),
		stationsBySystem,
	};
}

/**
 * What the planner reads off the map, plus how the static tables arrive. Narrow on
 * purpose, like [`LayoutHost`]: a test hands in a plain object.
 */
export interface RouteHost {
	mapId: number;
	systems(): MapSystemView[];
	connections(): MapConnection[];
	sigs(): Signature[];
	eveScout(): EveScoutConnection[];
	useEveScout(): boolean;
	/** Whether the chain's own holes count as route edges. */
	useWormholes(): boolean;
	/** The map's own ignore list, which every viewer routes around. */
	mapIgnored(): Set<number>;
	/** The viewer's route preference and tolerances. */
	settings(): RoutingSettings;
	loadTables(): Promise<RoutingTables>;
}

export class RoutePlanner {
	private map: RouteHost;

	// Origin/destination (solar system ids) and the computed path, set by the navigation
	// card. The path drives the edge highlight.
	fromId = $state<number | null>(null);
	toId = $state<number | null>(null);
	path = $state<number[]>([]);
	// A route hovered in a side panel temporarily replaces the pinned A→B highlight.
	hoverPath = $state<number[] | null>(null);
	// Systems this viewer steers around (per map, persisted locally).
	ignoredSystems = $state<Set<number>>(new Set());
	/** Everything the router steps around: the viewer's own list plus the map's. */
	avoided = $derived.by<Set<number>>(() => {
		const shared = this.map.mapIgnored();
		if (shared.size === 0) return this.ignoredSystems;
		return new Set([...this.ignoredSystems, ...shared]);
	});

	// The static routing data, fetched once and shared: the navigation card plans routes
	// with it, and the pilots card measures distances with it. One home, one fetch.
	stargates = $state<Map<number, number[]> | null>(null);
	security = $state<Map<number, number>>(new Map());
	joveSystems = $state<Set<number>>(new Set());
	stationSystems = $state<Set<number>>(new Set());
	/** A named set of stations to search for: what a station does, or who owns it. */
	serviceOptions = $state<StationGroup[]>([]);
	corporationOptions = $state<StationGroup[]>([]);

	constructor(map: RouteHost) {
		this.map = map;
	}

	// The last graph handed out and what it was built from. A memo, not state.
	private built: {
		stargates: Map<number, number[]>;
		security: Map<number, number>;
		edges: DynamicEdge[];
		graph: RouteGraph;
	} | null = null;

	/**
	 * Stargates plus the chain's own edges. `null` until the static data has arrived. Almost
	 * every refetch of the map hands over new arrays with the same edges in them (a renamed
	 * node, a moved one, a pasted scan), so the graph is only rebuilt when the edges it
	 * produces really differ; the same object otherwise, and every route search on it holds.
	 */
	graph = $derived.by<RouteGraph | null>(() => {
		const stargates = this.stargates;
		if (!stargates) return null;
		const security = this.security;
		const edges = graphBuild.chainEdges(
			this.map.systems(),
			this.map.connections(),
			this.map.useEveScout() ? this.map.eveScout() : null,
			this.map.useWormholes(),
		);
		const built = this.built;
		if (
			built &&
			built.stargates === stargates &&
			built.security === security &&
			graphBuild.sameEdges(built.edges, edges)
		) {
			return built.graph;
		}
		const graph = { stargates, dynamic: buildDynamicAdjacency(edges), security };
		this.built = { stargates, security, edges, graph };
		return graph;
	});

	private searches = new RouteSearchCache();

	/**
	 * Every route out of `origin` under the viewer's settings, avoiding what they avoid plus
	 * `extraIgnored`. Shared: every panel asking about the same origin gets the same search,
	 * run once. Reading it inside a derived tracks the graph, the settings and the avoided
	 * systems, so the derived reruns when any of them changes. Null until the tables are in.
	 */
	search(origin: number, extraIgnored: readonly number[] = []): RouteSearch | null {
		const graph = this.graph;
		if (!graph) return null;
		const avoided = this.avoided;
		const ignored = extraIgnored.length ? new Set([...avoided, ...extraIgnored]) : avoided;
		return this.searches.get(graph, origin, this.map.settings(), ignored);
	}

	connectionIds = $derived.by(() =>
		graphBuild.routeConnectionIds(this.path, this.map.systems(), this.map.connections()),
	);

	private ignoreStorageKey(): string {
		return `route-ignored-${this.map.mapId}`;
	}

	loadIgnored() {
		this.ignoredSystems = new Set(readStored(this.ignoreStorageKey(), v.array(v.number()), []));
	}

	ignoreSystem(id: number) {
		const next = new Set(this.ignoredSystems);
		next.add(id);
		this.ignoredSystems = next;
		writeStored(this.ignoreStorageKey(), [...next]);
	}

	clearIgnored() {
		this.ignoredSystems = new Set();
		removeStored(this.ignoreStorageKey());
	}

	/** The static routing tables (stargates, security, Jove/station/service indexes). */
	private routingLoad: Promise<void> | null = null;

	/**
	 * Resolves once the static routing tables are in. Jump tracking waits on this rather
	 * than reading `stargates` directly, since a jump taken seconds after the map opens
	 * would otherwise be judged against an empty gate table.
	 */
	whenLoaded(): Promise<void> {
		return this.routingLoad ?? this.load();
	}

	async load() {
		this.routingLoad ??= this.fetchGraph();
		return this.routingLoad;
	}

	private async fetchGraph() {
		try {
			const g = await this.map.loadTables();
			this.stargates = new Map(
				Object.entries(g.adjacency).map(([k, v]) => [Number(k), v as number[]]),
			);
			this.security = new Map(Object.entries(g.security).map(([k, v]) => [Number(k), v]));
			this.joveSystems = new Set(g.jove ?? []);
			this.stationSystems = new Set(g.stations ?? []);
			this.serviceOptions = (g.services ?? []).map(stationGroup);
			this.corporationOptions = (g.corporations ?? []).map(stationGroup);
		} catch {
			// No graph means no routing; the cards fall back to showing no distances.
		}
	}

	/** Route steps with the signature attached to each wormhole hop. */
	withSignatures(steps: RouteStep[]): (RouteStep & { signature: string | null })[] {
		return routeSignatures.withSignatures(
			steps,
			this.map.systems(),
			this.map.connections(),
			this.map.sigs(),
		);
	}
}
