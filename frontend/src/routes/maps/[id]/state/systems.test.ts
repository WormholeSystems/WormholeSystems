import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { GridConfig } from '$lib/api/types/GridConfig';
import type { MapSystemView } from '$lib/api/types/MapSystemView';
import type { MapAction } from '$lib/map/actions';

vi.mock('$lib/api/client', () => ({
	ApiError: class ApiError extends Error {
		constructor(
			public status: number,
			message: string,
		) {
			super(message);
		}
	},
	api: {
		addSystem: vi.fn(),
		addConnection: vi.fn(),
		setAlias: vi.fn(),
		setOccupier: vi.fn(),
		setNotes: vi.fn(),
	},
}));

const { api, ApiError } = await import('$lib/api/client');
const { SystemsApi } = await import('./systems');

const grid: GridConfig = {
	cell_size: 20,
	world_width: 4000,
	world_height: 2000,
	viewport_height: 1400,
};

const MAP_ID = 5;
const C13 = 13;

function setup(systems: MapSystemView[], opts: { runNow?: boolean } = {}) {
	const runs: { action: MapAction; exec: () => Promise<unknown> }[] = [];
	const echoed: (string | null)[] = [];
	const refetched: number[] = [];
	const host = {
		mapId: MAP_ID,
		camera: {
			pan: { x: 0, y: 0 },
			zoom: 1,
			viewportRect: () => ({ left: 0, top: 0, width: 800, height: 600 }),
		},
		all: () => systems,
		byId: () => new Map(systems.map((s) => [s.id, s])),
		orphaned: () => [],
		grid: () => grid,
		run: <T>(action: MapAction, exec: () => Promise<T>) => {
			runs.push({ action, exec });
			return opts.runNow ? exec() : Promise.resolve(undefined as T);
		},
		notesLocal: (_id: number, notes: string | null) => echoed.push(notes),
		refetchNotes: (id: number) => refetched.push(id),
	};
	// The host's camera is a structural fake; the api only reads pan, zoom and the rect.
	const systemsApi = new SystemsApi(host as unknown as ConstructorParameters<typeof SystemsApi>[0]);
	return { systemsApi, runs, echoed, refetched };
}

const placed = (id: number, over: Partial<MapSystemView> = {}) =>
	({
		kind: 'system',
		id,
		solar_system_id: 30000000 + id,
		position_x: 0,
		position_y: 0,
		wormhole_class_id: null,
		security_status: 0.5,
		occupying_group: 'Old Group',
		...over,
	}) as MapSystemView;

beforeEach(() => vi.clearAllMocks());

describe('SystemsApi.add', () => {
	it('hands the write over unstarted, and places the system at a free spot', async () => {
		vi.mocked(api.addSystem).mockResolvedValue({ id: 9 } as never);
		const { systemsApi, runs } = setup([]);

		systemsApi.add(30000142, { anchor: { x: 300, y: 200 } });
		expect(api.addSystem).not.toHaveBeenCalled();

		expect(runs.map((r) => r.action)).toEqual(['addSystem']);
		await runs[0].exec();
		expect(api.addSystem).toHaveBeenCalledWith({
			map_id: MAP_ID,
			solar_system_id: 30000142,
			x: 300,
			y: 200,
			alias: null,
		});
		expect(api.addConnection).not.toHaveBeenCalled();
	});

	it('connects the new placement to the one it was added from, sized by the pair', async () => {
		vi.mocked(api.addSystem).mockResolvedValue({ id: 9 } as never);
		const from = placed(1, { wormhole_class_id: C13 });
		const { systemsApi, runs } = setup([from, placed(9)]);

		systemsApi.add(30000009, { connectFrom: from.id });
		await runs[0].exec();
		expect(api.addConnection).toHaveBeenCalledWith({
			map_id: MAP_ID,
			from_system: 1,
			to_system: 9,
			kind: 'wormhole',
			size: 'small',
		});
	});

	it('does not connect a system to itself when the add merged into the source', async () => {
		vi.mocked(api.addSystem).mockResolvedValue({ id: 1 } as never);
		const { systemsApi, runs } = setup([placed(1)]);
		systemsApi.add(30000001, { connectFrom: 1 });
		await runs[0].exec();
		expect(api.addConnection).not.toHaveBeenCalled();
	});
});

describe('SystemsApi.rename', () => {
	it('writes the alias and the occupier of a real system as one action', async () => {
		const { systemsApi, runs } = setup([]);
		systemsApi.rename(placed(3), 'A1', 'New Group');
		expect(api.setAlias).not.toHaveBeenCalled();

		expect(runs.map((r) => r.action)).toEqual(['setAlias']);
		await runs[0].exec();
		expect(api.setAlias).toHaveBeenCalledWith({
			map_id: MAP_ID,
			map_solar_system_id: 3,
			alias: 'A1',
		});
		expect(api.setOccupier).toHaveBeenCalledWith({
			map_id: MAP_ID,
			map_solar_system_id: 3,
			occupier: 'New Group',
		});
	});

	it('leaves the occupier alone on a ghost, which has none', async () => {
		const { systemsApi, runs } = setup([]);
		systemsApi.rename({ kind: 'ghost', id: 4 } as MapSystemView, 'A2', null);
		await runs[0].exec();
		expect(api.setAlias).toHaveBeenCalledOnce();
		expect(api.setOccupier).not.toHaveBeenCalled();
	});
});

describe('SystemsApi.saveNotes', () => {
	it('sends the notes the editor started from, and echoes the new ones', async () => {
		vi.mocked(api.setNotes).mockResolvedValue(null);
		const { systemsApi, echoed, refetched } = setup([], { runNow: true });

		expect(await systemsApi.saveNotes(3, 'frig hole', 'static rolled')).toBe('saved');
		expect(api.setNotes).toHaveBeenCalledWith({
			map_id: MAP_ID,
			map_solar_system_id: 3,
			notes: 'frig hole',
			expected: 'static rolled',
		});
		expect(echoed).toEqual(['frig hole']);
		expect(refetched).toEqual([]);
	});

	it('says a 409 is a conflict, not a failure, and reads theirs back', async () => {
		vi.mocked(api.setNotes).mockRejectedValue(new ApiError(409, 'someone else edited'));
		const { systemsApi, runs, refetched } = setup([], { runNow: true });

		expect(await systemsApi.saveNotes(3, 'frig hole', null)).toBe('conflict');
		// Resolved inside the write, so the shared mutation does not toast it.
		await expect(runs[0].exec()).resolves.toBe('conflict');
		expect(refetched).toEqual([3]);
	});

	it('reads the notes back after any other failure too', async () => {
		vi.mocked(api.setNotes).mockRejectedValue(new ApiError(500, 'boom'));
		const { systemsApi, refetched } = setup([], { runNow: true });

		expect(await systemsApi.saveNotes(3, 'frig hole', null)).toBe('failed');
		expect(refetched).toEqual([3]);
	});
});
