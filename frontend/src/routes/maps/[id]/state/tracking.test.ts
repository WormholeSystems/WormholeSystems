import { describe, expect, it, vi } from 'vitest';

import type { CharacterRef } from '$lib/api/types/CharacterRef';
import type { MapConnection } from '$lib/api/types/MapConnection';
import type { MapSystemView } from '$lib/api/types/MapSystemView';
import type { Signature } from '$lib/api/types/Signature';
import type { SignatureCatalog } from '$lib/api/types/SignatureCatalog';
import { JumpTracker, type TrackerHost } from './tracking.svelte';

const GRID = { cell_size: 20, world_width: 4000, world_height: 2000, viewport_height: 1400 };
const CATALOG = { types: [] } as unknown as SignatureCatalog;

const system = (id: number, solarSystemId: number, over: Partial<MapSystemView> = {}) =>
	({
		kind: 'system',
		id,
		solar_system_id: solarSystemId,
		name: `J${solarSystemId}`,
		alias: null,
		position_x: 200,
		position_y: 200,
		is_pinned: false,
		wormhole_class_id: 5,
		security_status: -1,
		...over,
	}) as MapSystemView;
const ghost = (id: number): MapSystemView =>
	({ kind: 'ghost', id, alias: null, position_x: 400, position_y: 200 }) as MapSystemView;
const conn = (id: number, from: number, to: number): MapConnection =>
	({ id, from_system: from, to_system: to, kind: 'wormhole' }) as MapConnection;
/** A hole scanned in the origin and not yet linked to anything. */
const scanned = {
	id: 5,
	signature_id: 'ABC-123',
	solar_system_id: 100,
	group: 'wormhole',
	signature_type_id: null,
	connection_id: null,
} as Signature;

function fakeHost(over: Partial<TrackerHost> = {}) {
	const trackJump = vi.fn();
	const resolveGhost = vi.fn();
	let myCharacters: CharacterRef[] = [];
	const host: TrackerHost = {
		myCharacters: () => myCharacters,
		systems: () => [system(1, 100)],
		connections: () => [],
		sigs: () => [],
		grid: () => GRID,
		settings: () => ({
			tracking_allowed: true,
			prompt_for_signature: true,
			suggest_alias: false,
			tracked_character_ids: [],
		}),
		naming: () => null,
		ignored: () => new Set(),
		stargates: () => new Map(),
		whenRoutingLoaded: () => Promise.resolve(),
		loadCatalog: () => Promise.resolve(CATALOG),
		resolveSystem: (id) =>
			Promise.resolve({
				id,
				name: `J${id}`,
				security: -1,
				region: 'D-R00018',
				region_id: 0,
				constellation_id: 0,
				wormhole_class_id: 5,
				effect_name: null,
				sovereignty: null,
				statics: [],
			}),
		trackJump,
		resolveGhost,
		...over,
	};
	const setPilots = (pilots: CharacterRef[]) => {
		myCharacters = pilots;
	};
	return { host, trackJump, resolveGhost, setPilots };
}

/** Feed the tracker two readings of one pilot: the baseline, then the arrival. */
async function fly(
	tracker: JumpTracker,
	setPilots: (pilots: CharacterRef[]) => void,
	from: number,
	to: number,
) {
	setPilots([
		{ character_id: 9, is_active: true, online: true, solar_system_id: from } as CharacterRef,
	]);
	tracker.observe();
	setPilots([
		{ character_id: 9, is_active: true, online: true, solar_system_id: to } as CharacterRef,
	]);
	tracker.observe();
	// The decision path awaits routing tables and the catalog.
	await Promise.resolve();
	await Promise.resolve();
	await Promise.resolve();
	await Promise.resolve();
}

describe('JumpTracker', () => {
	it('maps an unscanned hole without asking', async () => {
		const { host, trackJump, setPilots } = fakeHost();
		const tracker = new JumpTracker(host);
		await fly(tracker, setPilots, 100, 200);
		expect(trackJump).toHaveBeenCalledOnce();
		expect(trackJump.mock.calls[0][0]).toMatchObject({
			from_map_solar_system_id: 1,
			to_solar_system_id: 200,
		});
		expect(tracker.prompt).toBeNull();
	});

	it('flies through an ignored system without mapping it', async () => {
		const { host, trackJump, setPilots } = fakeHost({ ignored: () => new Set([200]) });
		const tracker = new JumpTracker(host);
		await fly(tracker, setPilots, 100, 200);
		expect(trackJump).not.toHaveBeenCalled();
		expect(tracker.prompt).toBeNull();
	});

	it('still records a jump into an ignored system somebody placed by hand', async () => {
		const { host, trackJump, setPilots } = fakeHost({
			systems: () => [system(1, 100), system(2, 200)],
			ignored: () => new Set([200]),
			settings: () => ({
				tracking_allowed: true,
				prompt_for_signature: false,
				suggest_alias: false,
				tracked_character_ids: [],
			}),
		});
		const tracker = new JumpTracker(host);
		await fly(tracker, setPilots, 100, 200);
		expect(trackJump).toHaveBeenCalledOnce();
	});

	it('maps every tracked pilot, and only them', async () => {
		const { host, trackJump, setPilots } = fakeHost({
			settings: () => ({
				tracking_allowed: true,
				prompt_for_signature: true,
				suggest_alias: false,
				tracked_character_ids: [9, 10],
			}),
		});
		const tracker = new JumpTracker(host);
		const at = (a: number, b: number, c: number) =>
			setPilots([
				{ character_id: 9, name: 'Main', is_active: true, online: true, solar_system_id: a },
				{ character_id: 10, name: 'Alt', is_active: false, online: true, solar_system_id: b },
				{ character_id: 11, name: 'Other', is_active: false, online: true, solar_system_id: c },
			] as CharacterRef[]);
		at(100, 100, 100);
		tracker.observe();
		at(200, 300, 400);
		tracker.observe();
		for (let i = 0; i < 4; i++) await Promise.resolve();
		expect(trackJump).toHaveBeenCalledTimes(2);
		expect(trackJump.mock.calls.map((c) => c[0].to_solar_system_id).sort()).toEqual([200, 300]);
	});

	it('asks about one jump at a time, and the next once the first is answered', async () => {
		const { host, setPilots } = fakeHost({
			sigs: () => [
				{
					id: 5,
					signature_id: 'ABC-123',
					solar_system_id: 100,
					group: 'wormhole',
					signature_type_id: null,
					connection_id: null,
				} as Signature,
			],
			settings: () => ({
				tracking_allowed: true,
				prompt_for_signature: true,
				suggest_alias: false,
				tracked_character_ids: [9, 10],
			}),
		});
		const tracker = new JumpTracker(host);
		const at = (a: number, b: number) =>
			setPilots([
				{ character_id: 9, name: 'Main', is_active: true, online: true, solar_system_id: a },
				{ character_id: 10, name: 'Alt', is_active: false, online: true, solar_system_id: b },
			] as CharacterRef[]);
		at(100, 100);
		tracker.observe();
		at(200, 300);
		tracker.observe();
		for (let i = 0; i < 6; i++) await Promise.resolve();
		expect(tracker.prompt).toMatchObject({ pilot: 'Main', targetSolarSystemId: 200 });
		tracker.dismiss();
		for (let i = 0; i < 6; i++) await Promise.resolve();
		expect(tracker.prompt).toMatchObject({ pilot: 'Alt', targetSolarSystemId: 300 });
		tracker.dismiss();
		expect(tracker.prompt).toBeNull();
	});

	it('maps a jump left without a signature, unlinked', async () => {
		const { host, trackJump, setPilots } = fakeHost({ sigs: () => [scanned] });
		const tracker = new JumpTracker(host);
		await fly(tracker, setPilots, 100, 200);
		const prompt = tracker.prompt!;
		expect(prompt).not.toBeNull();

		// What closing the prompt sends: no signature, and whatever else was filled in.
		tracker.submit({
			origin: prompt.origin,
			targetSolarSystemId: prompt.targetSolarSystemId,
			signaturePk: null,
			alias: '1',
			at: prompt.at,
			timeStatus: 'eol',
		});
		expect(trackJump).toHaveBeenCalledOnce();
		expect(trackJump.mock.calls[0][0]).toMatchObject({
			from_map_solar_system_id: 1,
			to_solar_system_id: 200,
			alias: '1',
			time_status: 'eol',
		});
		expect(trackJump.mock.calls[0][0].signature_pk).toBeUndefined();
	});

	it('leaves a hole already on the map alone when no signature is picked', async () => {
		const { host, trackJump, setPilots } = fakeHost({
			systems: () => [system(1, 100), system(2, 200)],
			connections: () => [conn(10, 1, 2)],
			sigs: () => [scanned],
		});
		const tracker = new JumpTracker(host);
		await fly(tracker, setPilots, 100, 200);
		// Asked, because the connection says nothing yet about which signature it was.
		const prompt = tracker.prompt!;
		expect(prompt).not.toBeNull();

		tracker.submit({
			origin: prompt.origin,
			targetSolarSystemId: prompt.targetSolarSystemId,
			signaturePk: null,
			alias: null,
			at: prompt.at,
		});
		expect(trackJump).not.toHaveBeenCalled();
	});

	it('suppresses a gate hop', async () => {
		const { host, trackJump, setPilots } = fakeHost({ stargates: () => new Map([[100, [200]]]) });
		const tracker = new JumpTracker(host);
		await fly(tracker, setPilots, 100, 200);
		expect(trackJump).not.toHaveBeenCalled();
	});

	it('does nothing while tracking is off, but still keeps its baseline', async () => {
		const { host, trackJump, setPilots } = fakeHost({
			settings: () => ({
				tracking_allowed: false,
				prompt_for_signature: true,
				suggest_alias: false,
				tracked_character_ids: [],
			}),
		});
		const tracker = new JumpTracker(host);
		await fly(tracker, setPilots, 100, 200);
		expect(trackJump).not.toHaveBeenCalled();
	});

	it('resolves the lone ghost instead of mapping the system twice', async () => {
		const { host, trackJump, resolveGhost, setPilots } = fakeHost({
			systems: () => [system(1, 100), ghost(2)],
			connections: () => [conn(10, 1, 2)],
		});
		const tracker = new JumpTracker(host);
		await fly(tracker, setPilots, 100, 200);
		expect(resolveGhost).toHaveBeenCalledOnce();
		expect(resolveGhost.mock.calls[0][0]).toMatchObject({
			map_solar_system_id: 2,
			solar_system_id: 200,
		});
		expect(trackJump).not.toHaveBeenCalled();
	});

	it('records nothing for an already-mapped, already-explained connection', async () => {
		const { host, trackJump, setPilots } = fakeHost({
			systems: () => [system(1, 100), system(2, 200)],
			connections: () => [conn(10, 1, 2)],
			sigs: () => [{ id: 7, connection_id: 10, solar_system_id: 100 } as Signature],
		});
		const tracker = new JumpTracker(host);
		await fly(tracker, setPilots, 100, 200);
		expect(trackJump).not.toHaveBeenCalled();
		expect(tracker.prompt).toBeNull();
	});

	it('ignores a jump that does not leave the mapped chain', async () => {
		const { host, trackJump, setPilots } = fakeHost();
		const tracker = new JumpTracker(host);
		await fly(tracker, setPilots, 999, 200);
		expect(trackJump).not.toHaveBeenCalled();
	});
});
