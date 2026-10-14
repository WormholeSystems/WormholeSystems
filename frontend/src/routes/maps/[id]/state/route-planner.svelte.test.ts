import { flushSync } from 'svelte';
import { describe, expect, it } from 'vitest';

import type { MapConnection } from '$lib/api/types/MapConnection';
import type { MapSystemView } from '$lib/api/types/MapSystemView';
import type { RoutingGraph } from '$lib/api/types/RoutingGraph';
import { RoutePlanner } from './route-planner.svelte';

const TABLES: RoutingGraph = {
	adjacency: { '100': [200], '200': [100] },
	security: { '100': 0.9, '200': 0.4 },
	jove: [],
	stations: [],
	services: [],
	corporations: [],
};

const placed = (id: number, solar: number, over: Partial<MapSystemView> = {}) =>
	({ kind: 'system', id, solar_system_id: solar, alias: null, ...over }) as MapSystemView;

const hole = (id: number, from: number, to: number, over: Partial<MapConnection> = {}) =>
	({
		id,
		kind: 'wormhole',
		from_system: from,
		to_system: to,
		mass_status: null,
		time_status: null,
		...over,
	}) as MapConnection;

describe('RoutePlanner graph', () => {
	it('keeps the same graph across a refetch that changed nothing it routes over', async () => {
		let systems = $state<MapSystemView[]>([placed(1, 100), placed(2, 300)]);
		let connections = $state<MapConnection[]>([hole(9, 1, 2)]);
		const planner = new RoutePlanner({
			mapId: 1,
			systems: () => systems,
			connections: () => connections,
			sigs: () => [],
			eveScout: () => [],
			useEveScout: () => false,
			useWormholes: () => true,
			mapIgnored: () => new Set(),
			settings: () => ({
				preference: 'shorter',
				securityPenalty: 50,
				allowTimeStatus: 'critical',
				allowMassStatus: 'reduced',
			}),
			loadTables: () => Promise.resolve(TABLES),
		});
		await planner.load();
		const first = planner.graph;
		expect(first).not.toBeNull();

		// A rename comes back as new arrays with the same edges in them.
		systems = [placed(1, 100, { alias: 'Home' }), placed(2, 300)];
		connections = [hole(9, 1, 2)];
		flushSync();
		expect(planner.graph).toBe(first);

		// A mass change is one the router reads.
		connections = [hole(9, 1, 2, { mass_status: 'critical' })];
		flushSync();
		expect(planner.graph).not.toBe(first);
	});
});
