import { describe, expect, it, vi } from 'vitest';

import type { MapSystemView } from '$lib/api/types/MapSystemView';
import { MapCamera } from './map-camera.svelte';
import { MapGestures, type GestureHost } from './map-gestures.svelte';

const GRID = { cell_size: 20, world_width: 4000, world_height: 2000, viewport_height: 1400 };

const system = (id: number, x: number, y: number, over: Partial<MapSystemView> = {}) =>
	({
		kind: 'system',
		id,
		position_x: x,
		position_y: y,
		is_pinned: false,
		...over,
	}) as MapSystemView;
const ghost = (id: number, x: number, y: number): MapSystemView =>
	({ kind: 'ghost', id, position_x: x, position_y: y, is_pinned: false }) as MapSystemView;

function pointer(clientX: number, clientY: number, button = 0): PointerEvent {
	return {
		clientX,
		clientY,
		button,
		pointerId: 1,
		shiftKey: false,
		ctrlKey: false,
		metaKey: false,
		preventDefault: vi.fn(),
		stopPropagation: vi.fn(),
	} as unknown as PointerEvent;
}

function fakeHost(systems: MapSystemView[], over: Partial<GestureHost> = {}) {
	const camera = new MapCamera(1);
	camera.viewportSize = { width: 800, height: 600 };
	const move = vi.fn();
	const connect = vi.fn();
	const host: GestureHost = {
		camera,
		systems: { all: systems, move },
		positions: new Map(systems.map((s) => [s.id, { x: s.position_x, y: s.position_y }])),
		nodeH: 40,
		grid: GRID,
		canWrite: true,
		layoutLocked: false,
		selected: new Set<number>(),
		drag: null,
		pending: {},
		linking: null,
		band: null,
		panDrag: null,
		snap: (v) => Math.round(v / GRID.cell_size) * GRID.cell_size,
		clampNodeX: (x) => x,
		clampNodeY: (y) => y,
		closeMenu: vi.fn(),
		connections: { add: connect },
		...over,
	};
	return { host, move, connect };
}

describe('MapGestures', () => {
	it('keeps a grab a tap under the hysteresis, then commits the drag past it', () => {
		const { host } = fakeHost([system(1, 100, 100)]);
		const gestures = new MapGestures(host);
		gestures.onNodeDown(pointer(50, 50), host.systems.all[0]);
		gestures.onPointerMove(pointer(53, 50));
		expect(host.drag).toBeNull();
		gestures.onPointerMove(pointer(54, 50));
		expect(host.drag).not.toBeNull();
		expect(host.drag?.primary).toBe(1);
	});

	it('seeds the optimistic override and issues one move on drop', () => {
		const { host, move } = fakeHost([system(1, 100, 100)]);
		const gestures = new MapGestures(host);
		gestures.onNodeDown(pointer(50, 50), host.systems.all[0]);
		gestures.onPointerMove(pointer(150, 50));
		gestures.onPointerUp(pointer(150, 50));
		expect(move).toHaveBeenCalledOnce();
		const moves = move.mock.calls[0][0];
		expect(moves[0].map_solar_system_id).toBe(1);
		expect(host.pending[1]).toEqual({ x: moves[0].x, y: moves[0].y });
		expect(host.drag).toBeNull();
	});

	it('connects a link drop on a real system, but never on a ghost', () => {
		const systems = [system(1, 0, 0), system(2, 200, 0), ghost(3, 400, 0)];
		const { host, connect } = fakeHost(systems);
		const gestures = new MapGestures(host);

		host.linking = { from: 1, x: 0, y: 0 };
		gestures.onPointerUp(pointer(210, 10));
		expect(connect).toHaveBeenCalledWith(1, 2);

		host.linking = { from: 1, x: 0, y: 0 };
		gestures.onPointerUp(pointer(410, 10));
		expect(connect).toHaveBeenCalledOnce();
	});

	it('clears the selection on a background tap that never became a band', () => {
		const { host } = fakeHost([system(1, 100, 100)]);
		host.selected = new Set([1]);
		const gestures = new MapGestures(host);
		gestures.onBackgroundDown(pointer(300, 300));
		gestures.onPointerUp(pointer(301, 300));
		expect(host.selected.size).toBe(0);
	});

	it('pans instead of banding when the layout is locked', () => {
		const { host } = fakeHost([system(1, 100, 100)], { layoutLocked: true });
		const gestures = new MapGestures(host);
		gestures.onBackgroundDown(pointer(300, 300));
		expect(host.panDrag).not.toBeNull();
		gestures.onPointerMove(pointer(320, 310));
		expect(host.camera.pan).toEqual({ x: 20, y: 10 });
	});

	it('refuses a node drag without write access', () => {
		const { host } = fakeHost([system(1, 100, 100)], { canWrite: false });
		const gestures = new MapGestures(host);
		gestures.onNodeDown(pointer(50, 50), host.systems.all[0]);
		gestures.onPointerMove(pointer(150, 50));
		expect(host.drag).toBeNull();
	});

	it('leaves the drag alone while the pointer stays inside one grid cell', () => {
		const { host } = fakeHost([system(1, 100, 100)]);
		const gestures = new MapGestures(host);
		gestures.onNodeDown(pointer(50, 50), host.systems.all[0]);
		gestures.onPointerMove(pointer(100, 50));
		const before = host.drag;
		gestures.onPointerMove(pointer(103, 52));
		expect(host.drag).toBe(before);
		gestures.onPointerMove(pointer(130, 50));
		expect(host.drag).not.toBe(before);
		expect(host.drag?.x).toBe(180);
	});
});
