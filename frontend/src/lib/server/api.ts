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

import type { CharacterSummary } from '$lib/api/types/CharacterSummary';
import type { Instance } from '$lib/api/types/Instance';
import type { ReferenceCounts } from '$lib/api/types/ReferenceCounts';
import type { ServerStatus } from '$lib/api/types/ServerStatus';
import type { MapEntry } from '$lib/api/types/MapEntry';
import type { MapUserSettings } from '$lib/api/types/MapUserSettings';
import type { MapView } from '$lib/api/types/MapView';

const base = () => env.API_BASE ?? 'http://127.0.0.1:3000';

async function get<T>(event: RequestEvent, path: string): Promise<T> {
	const res = await event.fetch(`${base()}${path}`, {
		headers: { cookie: event.request.headers.get('cookie') ?? '' },
	});
	if (!res.ok) throw new Error(`API ${path} failed: ${res.status}`);
	return res.json();
}

/** The signed-in character, or null. Used by the layout and the /maps auth gate. */
export function currentCharacter(event: RequestEvent): Promise<CharacterSummary | null> {
	return get<CharacterSummary | null>(event, '/api/me');
}

/**
 * The map a share link leads to, or null. A withdrawn token and a mistyped one both come back
 * as not found, which is the point of a secret in a URL.
 */
export async function sharedMapId(event: RequestEvent, token: string): Promise<number | null> {
	try {
		const view = await get<MapView>(event, `/api/share/${encodeURIComponent(token)}`);
		return view.map.id;
	} catch {
		return null;
	}
}

/** A map's graph and the caller's role on it, for the settings pages. */
export function mapView(event: RequestEvent, mapId: number): Promise<MapView> {
	return get<MapView>(event, `/api/maps/${mapId}`);
}

/** Every map the caller can see. */
export function myMaps(event: RequestEvent): Promise<MapEntry[]> {
	return get<MapEntry[]>(event, '/api/maps');
}

/** The caller's own settings for one map. */
export function mapUserSettings(event: RequestEvent, mapId: number): Promise<MapUserSettings> {
	return get<MapUserSettings>(event, `/api/maps/${mapId}/settings/user`);
}

/**
 * How much static data this install has seeded, for the landing page. A failure falls back
 * to zeroes, which the page renders as a row it simply does not draw.
 */
export function referenceCounts(event: RequestEvent): Promise<ReferenceCounts> {
	return get<ReferenceCounts>(event, '/api/reference-counts').catch(() => ({
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
	return get<ServerStatus>(event, '/api/server-status').catch(() => null);
}

/**
 * What this deployment has switched on, and its operator's banner. Null when the API does not
 * answer: the page is better off without a banner than without a page.
 */
export function instance(event: RequestEvent): Promise<Instance | null> {
	return get<Instance>(event, '/api/instance').catch(() => null);
}
