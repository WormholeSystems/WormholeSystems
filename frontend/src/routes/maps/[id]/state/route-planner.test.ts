import { describe, expect, it } from 'vitest';

import type { RoutingGraph } from '$lib/api/types/RoutingGraph';
import type { RoutingSettings } from '$lib/routing/algorithm';
import { RoutePlanner, type RouteHost } from './route-planner.svelte';

const TABLES: RoutingGraph = {
	adjacency: { '100': [200], '200': [100] },
	security: { '100': 0.9, '200': 0.4 },
	jove: [300],
	stations: [100],
	services: [{ id: 1, name: 'Cloning', stations: [{ id: 5, name: 'Home', solar_system_id: 100 }] }],
	corporations: [],
};

const SETTINGS: RoutingSettings = {
	preference: 'shorter',
	securityPenalty: 50,
	allowTimeStatus: 'critical',
	allowMassStatus: 'reduced',
};

function host(over: Partial<RouteHost> = {}): RouteHost {
	return {
		mapId: 1,
		systems: () => [],
		connections: () => [],
		sigs: () => [],
		eveScout: () => [],
		useEveScout: () => false,
		useWormholes: () => true,
		mapIgnored: () => new Set<number>(),
		settings: () => SETTINGS,
		loadTables: () => Promise.resolve(TABLES),
		...over,
	};
}

describe('RoutePlanner', () => {
	it('has no graph until the tables arrive', async () => {
		const planner = new RoutePlanner(host());
		expect(planner.graph).toBeNull();
		await planner.load();
		expect(planner.graph).not.toBeNull();
		expect(planner.stargates?.get(100)).toEqual([200]);
		expect(planner.joveSystems.has(300)).toBe(true);
		expect(planner.serviceOptions[0]?.systems.has(100)).toBe(true);
	});

	it('fetches the tables once however many callers wait on it', async () => {
		let calls = 0;
		const planner = new RoutePlanner(
			host({
				loadTables: () => {
					calls += 1;
					return Promise.resolve(TABLES);
				},
			}),
		);
		await Promise.all([planner.whenLoaded(), planner.load(), planner.whenLoaded()]);
		expect(calls).toBe(1);
	});

	it('degrades to no routing when the tables fail to load', async () => {
		const planner = new RoutePlanner(host({ loadTables: () => Promise.reject(new Error('down')) }));
		await planner.load();
		expect(planner.graph).toBeNull();
		expect(planner.stargates).toBeNull();
	});

	it('keeps the ignore list as a set that grows and clears', () => {
		const planner = new RoutePlanner(host());
		planner.ignoreSystem(100);
		planner.ignoreSystem(200);
		expect(planner.ignoredSystems).toEqual(new Set([100, 200]));
		planner.clearIgnored();
		expect(planner.ignoredSystems.size).toBe(0);
	});

	it('routes around the map-wide list as well as its own', () => {
		const planner = new RoutePlanner(host({ mapIgnored: () => new Set([300]) }));
		planner.ignoreSystem(100);
		expect(planner.avoided).toEqual(new Set([100, 300]));
		planner.clearIgnored();
		expect(planner.avoided).toEqual(new Set([300]));
	});

	it('shares one search between everyone asking about the same origin', async () => {
		const planner = new RoutePlanner(host());
		expect(planner.search(100)).toBeNull();
		await planner.load();
		const search = planner.search(100);
		expect(search?.routeTo(200)?.jumps).toBe(1);
		expect(planner.search(100)).toBe(search);
		expect(planner.search(200)).not.toBe(search);
		expect(planner.search(100, [200])).not.toBe(search);
		expect(planner.search(100, [200])?.routeTo(200)?.jumps).toBe(1);
	});
});
