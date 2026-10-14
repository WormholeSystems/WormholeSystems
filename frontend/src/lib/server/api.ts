// Server-side API access for load functions: talks to the Axum backend directly (it is a
// separate service, so relative paths don't work here) and forwards the session cookie.
//
// Where data comes from, so the two halves stay apart:
//
//   - Anything the first painted frame needs comes from a load, and is refreshed by
//     invalidating its key. That is every page you navigate to.
//   - Anything a socket changes lives in `MapState` and refetches itself. That is the map
//     page, where the graph changes many times a minute without anybody navigating, and
//     where the refetches are deliberately partial.
//
// The map page uses both: its load seeds `MapState` so the first frame has the chain, and
// `MapState` takes over from there.

import { env } from '$env/dynamic/private';
import type { RequestEvent } from '@sveltejs/kit';

import { createApi } from '$lib/api/client';
import type { CharacterSummary } from '$lib/api/types/CharacterSummary';
import type { GridConfig } from '$lib/api/types/GridConfig';
import type { Instance } from '$lib/api/types/Instance';
import type { ReferenceCounts } from '$lib/api/types/ReferenceCounts';
import type { ServerStatus } from '$lib/api/types/ServerStatus';
import type { MapEntry } from '$lib/api/types/MapEntry';
import type { MapUserSettings } from '$lib/api/types/MapUserSettings';
import type { MapView } from '$lib/api/types/MapView';
import type { Signature } from '$lib/api/types/Signature';

const base = () => env.API_BASE ?? 'http://127.0.0.1:3000';

/**
 * The browser's typed client, pointed at the backend and carrying the visitor's session.
 * Failures are the same `ApiError`, status included, so a load can tell a map it may not
 * open from a backend that is down.
 */
function serverApi(event: RequestEvent) {
	return createApi(event.fetch, base(), { cookie: event.request.headers.get('cookie') ?? '' });
}

/** The signed-in character, or null. Used by the layout and the /maps auth gate. */
export function currentCharacter(event: RequestEvent): Promise<CharacterSummary | null> {
	return serverApi(event).me();
}

/**
 * The map a share link leads to, or null. A withdrawn token and a mistyped one both come back
 * as not found, which is the point of a secret in a URL.
 */
export async function sharedMapId(event: RequestEvent, token: string): Promise<number | null> {
	try {
		const view = await serverApi(event).sharedMap(token);
		return view.map.id;
	} catch {
		return null;
	}
}

/** A map's graph and the caller's role on it, for the settings pages. */
export function mapView(event: RequestEvent, mapId: number): Promise<MapView> {
	return serverApi(event).fetchMap(mapId);
}

/** Every map the caller can see. */
export function myMaps(event: RequestEvent): Promise<MapEntry[]> {
	return serverApi(event).myMaps();
}

/** The caller's own settings for one map. */
export function mapUserSettings(event: RequestEvent, mapId: number): Promise<MapUserSettings> {
	return serverApi(event).mapUserSettings(mapId);
}

/** Every signature on one map, which the signature panel filters per system. */
export function mapSignatures(event: RequestEvent, mapId: number): Promise<Signature[]> {
	return serverApi(event).listSignatures(mapId);
}

/** The canvas's grid and world size, which the map page draws with. */
export function gridConfig(event: RequestEvent): Promise<GridConfig> {
	return serverApi(event).gridConfig();
}

/**
 * How much static data this install has seeded, for the landing page. A failure falls back
 * to zeroes, which the page renders as a row it simply does not draw.
 */
export function referenceCounts(event: RequestEvent): Promise<ReferenceCounts> {
	return serverApi(event)
		.referenceCounts()
		.catch(() => ({
			solar_systems: 0,
			wormhole_systems: 0,
			stargates: 0,
			wormhole_types: 0,
		}));
}

/**
 * Tranquility's state for the top bar. A failure here is not worth a broken page: the
 * component polls anyway, and an unknown state is one it already knows how to draw.
 */
export function serverStatus(event: RequestEvent): Promise<ServerStatus | null> {
	return serverApi(event)
		.serverStatus()
		.catch(() => null);
}

/**
 * What this deployment has switched on, and its operator's banner. Null when the API does not
 * answer: the page is better off without a banner than without a page.
 */
export function instance(event: RequestEvent): Promise<Instance | null> {
	return serverApi(event)
		.instance()
		.catch(() => null);
}
