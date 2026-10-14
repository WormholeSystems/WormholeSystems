import { describe, expect, it } from 'vitest';

import type { MapConnection } from '$lib/api/types/MapConnection';
import type { MapSystemView } from '$lib/api/types/MapSystemView';
import {
	edgeTopologyKey,
	nodePositions,
	reusePositions,
	settlePending,
	treeShapeKey,
} from './positions';

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

const placed = (id: number, x: number, y: number) =>
	system(id, { position_x: x, position_y: y } as Partial<MapSystemView>);

describe('nodePositions', () => {
	const systems = [placed(1, 0, 0), placed(2, 100, 0), placed(3, 200, 0)];

	it('draws a drag over a pending move over the stored position', () => {
		const dragged = new Map([[1, { x: 5, y: 5 }]]);
		const pending = { 1: { x: 9, y: 9 }, 2: { x: 140, y: 40 } };
		const out = nodePositions(systems, null, dragged, pending);
		expect(out.get(1)).toEqual({ x: 5, y: 5 });
		expect(out.get(2)).toEqual({ x: 140, y: 40 });
		expect(out.get(3)).toEqual({ x: 200, y: 0 });
	});

	it('lets an automatic layout own every position, drag or not', () => {
		const layout = new Map([
			[1, { x: 0, y: 300 }],
			[2, { x: 0, y: 400 }],
		]);
		const out = nodePositions(systems, layout, new Map([[1, { x: 5, y: 5 }]]), {});
		expect(out).toEqual(layout);
		expect(out).not.toBe(layout);
	});
});

describe('settlePending', () => {
	it('drops a move once the server has it, within half a pixel', () => {
		const pending = { 1: { x: 40, y: 20 }, 2: { x: 300, y: 300 } };
		const systems = [placed(1, 40.4, 19.6), placed(2, 100, 0)];
		expect(settlePending(pending, systems)).toEqual({ 2: { x: 300, y: 300 } });
	});

	it('drops the move of a system that is gone', () => {
		expect(settlePending({ 7: { x: 1, y: 1 } }, [placed(1, 0, 0)])).toEqual({});
	});

	it('says nothing settled while the server still has the old position', () => {
		// An older payload landing after the drop must not snap the node back.
		const pending = { 1: { x: 40, y: 20 } };
		expect(settlePending(pending, [placed(1, 0, 0)])).toBeNull();
	});
});
