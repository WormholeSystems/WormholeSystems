import { describe, expect, it } from 'vitest';

import {
	RouteSearch,
	buildAdjacency,
	buildDynamicAdjacency,
	findClosestSystems,
	findRoute,
	findRoutes,
	type RouteGraph,
	type RoutingSettings,
} from './algorithm';
import { RouteSearchCache } from './search-cache';

const SHORTER: RoutingSettings = {
	preference: 'shorter',
	securityPenalty: 50,
	allowTimeStatus: 'critical',
	allowMassStatus: 'reduced',
};
const SAFER: RoutingSettings = { ...SHORTER, preference: 'safer' };

// 1 - 2 - 3 - 4 - 5 in a line, with a hole from 1 straight to 4 and a branch 2 - 6 - 7.
function graph(): RouteGraph {
	return {
		stargates: buildAdjacency([
			[1, 2],
			[2, 3],
			[3, 4],
			[4, 5],
			[2, 6],
			[6, 7],
		]),
		dynamic: buildDynamicAdjacency([{ a: 1, b: 4, via: 'wormhole', mass: null, time: null }]),
		security: new Map([
			[1, 0.9],
			[2, 0.5],
			[3, 0.1],
			[4, -0.2],
			[5, 0.6],
			[6, 0.8],
			[7, 0.3],
		]),
	};
}

describe('RouteSearch', () => {
	it('answers exactly as the one-off searches do', () => {
		const g = graph();
		for (const settings of [SHORTER, SAFER]) {
			for (const ignored of [new Set<number>(), new Set([3]), new Set([4, 6])]) {
				const search = new RouteSearch(g, 1, settings, ignored);
				for (const to of [1, 2, 3, 4, 5, 6, 7]) {
					expect(search.routeTo(to)).toEqual(findRoute(g, 1, to, settings, ignored));
				}
				expect(search.routesTo([5, 7, 2])).toEqual(findRoutes(g, 1, [5, 7, 2], settings, ignored));
				const odd = (id: number) => id % 2 === 1;
				for (const limit of [1, 2, 5]) {
					expect(search.closest(odd, limit)).toEqual(
						findClosestSystems(g, 1, odd, limit, settings, ignored),
					);
				}
			}
		}
	});

	it('still reaches a target that is itself avoided', () => {
		const search = new RouteSearch(graph(), 1, SHORTER, new Set([5]));
		expect(search.routeTo(5)?.jumps).toBe(2);
	});
});

describe('RouteSearchCache', () => {
	it('runs one search per origin, settings and avoided set', () => {
		const cache = new RouteSearchCache();
		const g = graph();
		const avoided = new Set([3]);
		const first = cache.get(g, 1, SHORTER, avoided);
		expect(cache.get(g, 1, { ...SHORTER }, new Set([3]))).toBe(first);
		expect(cache.get(g, 1, SAFER, avoided)).not.toBe(first);
		expect(cache.get(g, 2, SHORTER, avoided)).not.toBe(first);
		expect(cache.get(g, 1, SHORTER, new Set([3, 6]))).not.toBe(first);
	});

	it('forgets every search once the graph changes', () => {
		const cache = new RouteSearchCache();
		const first = cache.get(graph(), 1, SHORTER, new Set());
		expect(cache.get(graph(), 1, SHORTER, new Set())).not.toBe(first);
	});

	it('keeps only the most recent few', () => {
		const cache = new RouteSearchCache();
		const g = graph();
		const first = cache.get(g, 1, SHORTER, new Set());
		for (let from = 2; from <= 10; from++) cache.get(g, from, SHORTER, new Set());
		expect(cache.get(g, 1, SHORTER, new Set())).not.toBe(first);
	});
});
