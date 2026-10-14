// Arranging state through the API, and asserting that something does not happen. The
// request bodies are the generated wire types, so a field the server renames stops
// compiling here instead of quietly being ignored.

import { expect, type APIRequestContext, type Locator } from '@playwright/test';

import type { AddConnection } from '../src/lib/api/types/AddConnection';
import type { AddSystem } from '../src/lib/api/types/AddSystem';
import type { CreateMapBody } from '../src/lib/api/types/CreateMapBody';
import { createdId } from './fixtures';

export type Api = APIRequestContext;

/** A fresh map owned by whoever `api` is signed in as. */
export async function createMap(api: Api, name: string): Promise<number> {
	const res = await api.post('/api/maps', { data: { name } satisfies CreateMapBody });
	expect(res.ok()).toBe(true);
	return await createdId(res);
}

/** Where a placed system goes and what it is called; (200, 200) and no alias by default. */
export interface Placement {
	x?: number;
	y?: number;
	alias?: string | null;
}

/** Place a system on the map; answers with the placement id. */
export async function addSystem(
	api: Api,
	mapId: number,
	solarSystemId: number,
	{ x = 200, y = 200, alias = null }: Placement = {},
): Promise<number> {
	const res = await api.post(`/api/maps/${mapId}/systems/add`, {
		data: { map_id: mapId, solar_system_id: solarSystemId, x, y, alias } satisfies AddSystem,
	});
	expect(res.ok()).toBe(true);
	return await createdId(res);
}

/** Join two placements; a wormhole unless `extra` says otherwise. Answers with its id. */
export async function connect(
	api: Api,
	mapId: number,
	from: number,
	to: number,
	extra: Partial<Omit<AddConnection, 'map_id' | 'from_system' | 'to_system'>> = {},
): Promise<number> {
	const res = await api.post(`/api/maps/${mapId}/connections/add`, {
		data: {
			map_id: mapId,
			from_system: from,
			to_system: to,
			kind: 'wormhole',
			...extra,
		} satisfies AddConnection,
	});
	expect(res.ok()).toBe(true);
	return await createdId(res);
}

/**
 * Fails the moment `locator` shows up within `ms`. For asserting that something does not
 * happen: a plain negative assertion passes straight away, before it had the chance to.
 */
export async function staysHidden(locator: Locator, ms: number) {
	const appeared = await locator
		.first()
		.waitFor({ state: 'visible', timeout: ms })
		.then(
			() => true,
			() => false,
		);
	expect(appeared, `${locator} showed up within ${ms}ms`).toBe(false);
}

/** Reads `read` until `ms` have passed, failing the moment it is not `expected`. */
export async function holds<T>(read: () => Promise<T>, expected: T, ms: number, every = 250) {
	const until = Date.now() + ms;
	for (;;) {
		expect((await read()) as unknown).toEqual(expected);
		if (Date.now() >= until) return;
		await new Promise((resolve) => setTimeout(resolve, every));
	}
}
