import { describe, expect, it } from 'vitest';

import type { MapConnection } from '$lib/api/types/MapConnection';
import type { MapSystemView } from '$lib/api/types/MapSystemView';
import { edgeTopologyKey, reusePositions, treeShapeKey } from './positions';

const system = (id: number, over: Partial<MapSystemView> = {}) =>
	({
		kind: 'system',
		id,
		name: `S${id}`,
		alias: null,
		is_pinned: false,
		is_home: false,
		status: 'unknown',
		...over,
	}) as MapSystemView;

const connection = (id: number, from: number, to: number, over: Partial<MapConnection> = {}) =>
	({ id, from_system: from, to_system: to, mass_status: null, ...over }) as MapConnection;

describe('treeShapeKey', () => {
	it('ignores what the layout does not read', () => {
		const before = treeShapeKey([system(1), system(2)], [connection(9, 1, 2)]);
		const after = treeShapeKey(
			[system(1, { status: 'hostile' }), system(2)],
			[connection(9, 1, 2, { mass_status: 'critical' })],
		);
		expect(after).toBe(before);
	});

	it('changes with the nodes, the edges, the roots and the sort names', () => {
		const base = treeShapeKey([system(1), system(2)], [connection(9, 1, 2)]);
		expect(treeShapeKey([system(1)], [])).not.toBe(base);
		expect(treeShapeKey([system(1), system(2)], [connection(9, 2, 1)])).not.toBe(base);
		expect(
			treeShapeKey([system(1, { is_pinned: true }), system(2)], [connection(9, 1, 2)]),
		).not.toBe(base);
		expect(treeShapeKey([system(1, { alias: 'A' }), system(2)], [connection(9, 1, 2)])).not.toBe(
			base,
		);
	});
});

describe('reusePositions', () => {
	it('keeps the old object for an entry that did not move', () => {
		const a = { x: 0, y: 0 };
		const prev = new Map([
			[1, a],
			[2, { x: 20, y: 0 }],
		]);
		const next = reusePositions(
			new Map([
				[1, { x: 0, y: 0 }],
				[2, { x: 40, y: 0 }],
			]),
			prev,
		);
		expect(next).not.toBe(prev);
		expect(next.get(1)).toBe(a);
		expect(next.get(2)).toEqual({ x: 40, y: 0 });
	});

	it('hands back the previous map when nothing moved', () => {
		const prev = new Map([[1, { x: 0, y: 0 }]]);
		expect(reusePositions(new Map([[1, { x: 0, y: 0 }]]), prev)).toBe(prev);
	});

	it('does not treat a removed node as unchanged', () => {
		const prev = new Map([
			[1, { x: 0, y: 0 }],
			[2, { x: 20, y: 0 }],
		]);
		expect(reusePositions(new Map([[1, { x: 0, y: 0 }]]), prev)).not.toBe(prev);
	});
});

describe('edgeTopologyKey', () => {
	it('only follows which connections exist and what they join', () => {
		const key = edgeTopologyKey([connection(1, 1, 2)]);
		expect(edgeTopologyKey([connection(1, 1, 2, { mass_status: 'reduced' })])).toBe(key);
		expect(edgeTopologyKey([connection(1, 1, 3)])).not.toBe(key);
	});
});
