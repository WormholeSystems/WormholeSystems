import { describe, expect, it } from 'vitest';

import {
	ZARZAKH_SYSTEM_ID,
	buildAdjacency,
	buildDynamicAdjacency,
	findClosestSystems,
	findRoute,
	type DynamicEdge,
	type RouteGraph,
	type RoutingSettings,
} from './algorithm';

// A line of five stargate-linked systems, 1 through 5.
const LINE: [number, number][] = [
	[1, 2],
	[2, 3],
	[3, 4],
	[4, 5],
];

const settings = (over: Partial<RoutingSettings> = {}): RoutingSettings => ({
	preference: 'shorter',
	securityPenalty: 0,
	allowTimeStatus: 'critical',
	allowMassStatus: 'critical',
	...over,
});

const graph = (
	gates: [number, number][] = LINE,
	dynamic: DynamicEdge[] = [],
	security: [number, number][] = [],
): RouteGraph => ({
	stargates: buildAdjacency(gates),
	dynamic: buildDynamicAdjacency(dynamic),
	security: new Map(security),
});

const path = (result: { route: { id: number }[] } | null) => result?.route.map((s) => s.id);

describe('findRoute', () => {
	it('walks the gates when there is nothing else', () => {
		const found = findRoute(graph(), 1, 5, settings());
		expect(path(found)).toEqual([1, 2, 3, 4, 5]);
		expect(found?.jumps).toBe(4);
	});

	it('takes a wormhole that shortcuts the line', () => {
		const hole: DynamicEdge = { a: 1, b: 5, via: 'wormhole', mass: null, time: null };
		const found = findRoute(graph(LINE, [hole]), 1, 5, settings());
		expect(path(found)).toEqual([1, 5]);
		expect(found?.route[1].via).toBe('wormhole');
	});

	it('refuses a hole the tolerances rule out, and takes it once they allow it', () => {
		const dying: DynamicEdge = { a: 1, b: 5, via: 'wormhole', mass: 'critical', time: null };
		const strict = settings({ allowMassStatus: 'reduced' });
		expect(path(findRoute(graph(LINE, [dying]), 1, 5, strict))).toEqual([1, 2, 3, 4, 5]);
		expect(path(findRoute(graph(LINE, [dying]), 1, 5, settings()))).toEqual([1, 5]);
	});

	it('keeps an expired hole out at the default tolerance, and takes it once allowed', () => {
		const expired: DynamicEdge = { a: 1, b: 5, via: 'wormhole', mass: null, time: 'expired' };
		expect(path(findRoute(graph(LINE, [expired]), 1, 5, settings()))).toEqual([1, 2, 3, 4, 5]);
		const anything = settings({ allowTimeStatus: 'expired' });
		expect(path(findRoute(graph(LINE, [expired]), 1, 5, anything))).toEqual([1, 5]);
	});

	it('routes around a system the viewer has ignored', () => {
		const detour: [number, number][] = [...LINE, [2, 9], [9, 4]];
		const found = findRoute(graph(detour), 1, 5, settings(), new Set([3]));
		expect(path(found)).toEqual([1, 2, 9, 4, 5]);
	});

	it('never routes through Zarzakh, whatever it connects', () => {
		const viaZarzakh: [number, number][] = [
			[1, ZARZAKH_SYSTEM_ID],
			[ZARZAKH_SYSTEM_ID, 5],
		];
		expect(findRoute(graph(viaZarzakh), 1, 5, settings())).toBeNull();
	});

	it('is a zero-jump route to where you already are', () => {
		const found = findRoute(graph(), 3, 3, settings());
		expect(path(found)).toEqual([3]);
		expect(found?.jumps).toBe(0);
	});

	it('answers null when nothing connects the two', () => {
		expect(findRoute(graph([[1, 2]]), 1, 9, settings())).toBeNull();
	});
});

describe('route preferences', () => {
	// Three ways from 1 to 4, one per security band.
	const forked: [number, number][] = [
		[1, 2],
		[2, 4],
		[1, 7],
		[7, 4],
		[1, 8],
		[8, 4],
	];
	const sec: [number, number][] = [
		[1, 0.9],
		[2, -0.3],
		[7, 0.9],
		[8, 0.3],
		[4, 0.9],
	];

	it('shorter ignores security: every branch is two jumps', () => {
		expect(findRoute(graph(forked, [], sec), 1, 4, settings())?.jumps).toBe(2);
	});

	it('safer goes through highsec', () => {
		const found = findRoute(
			graph(forked, [], sec),
			1,
			4,
			settings({ preference: 'safer', securityPenalty: 50 }),
		);
		expect(path(found)).toEqual([1, 7, 4]);
	});

	// Less secure means lowsec, not nullsec: null is the most expensive band in both
	// preferences, so this never routes somebody through it to be contrarian.
	it('less_secure goes through lowsec and still avoids null', () => {
		const found = findRoute(
			graph(forked, [], sec),
			1,
			4,
			settings({ preference: 'less_secure', securityPenalty: 50 }),
		);
		expect(path(found)).toEqual([1, 8, 4]);
	});
});

describe('findClosestSystems', () => {
	it('returns matches nearest first and stops at the limit', () => {
		const found = findClosestSystems(graph(), 1, (id) => id > 2, 2, settings());
		expect(found.map((s) => s.id)).toEqual([3, 4]);
		expect(found[0].jumps).toBe(2);
	});
});
