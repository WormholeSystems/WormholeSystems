// One full route search per origin, shared by every panel that measures from it. The
// panels' deriveds still rerun whenever their own inputs change; what they no longer do is
// each pay for a Dijkstra over the whole universe when they do.

import { RouteSearch, type RouteGraph, type RoutingSettings } from './algorithm';

/** Enough for the shared origin, home, and a hub-less variant or two, with room to spare. */
const KEEP = 8;

function settingsKey(settings: RoutingSettings): string {
	return `${settings.preference}|${settings.securityPenalty}|${settings.allowTimeStatus}|${settings.allowMassStatus}`;
}

export class RouteSearchCache {
	private graph: RouteGraph | null = null;
	// Insertion order doubles as recency: a hit is moved to the back, the front is evicted.
	private searches = new Map<string, RouteSearch>();
	// Sets are compared by content, but building the key for one means sorting it, so the
	// key is kept per set object; the planner hands the same set over until it changes.
	private setKeys = new WeakMap<ReadonlySet<number>, string>();

	/** The search from `from` under these settings, run now unless one is already kept. */
	get(
		graph: RouteGraph,
		from: number,
		settings: RoutingSettings,
		ignored: ReadonlySet<number>,
	): RouteSearch {
		// A new graph object means the edges changed, and every kept search is stale.
		if (graph !== this.graph) {
			this.graph = graph;
			this.searches.clear();
		}
		const key = `${from}|${settingsKey(settings)}|${this.keyOf(ignored)}`;
		const kept = this.searches.get(key);
		if (kept) {
			this.searches.delete(key);
			this.searches.set(key, kept);
			return kept;
		}
		const search = new RouteSearch(graph, from, settings, ignored);
		this.searches.set(key, search);
		if (this.searches.size > KEEP) {
			const oldest = this.searches.keys().next().value;
			if (oldest !== undefined) this.searches.delete(oldest);
		}
		return search;
	}

	private keyOf(set: ReadonlySet<number>): string {
		let key = this.setKeys.get(set);
		if (key === undefined) {
			key = [...set].sort((a, b) => a - b).join(',');
			this.setKeys.set(set, key);
		}
		return key;
	}
}
