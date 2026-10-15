// What one frame off the map socket actually invalidates.
//
// The event says what changed and this believes it, rather than reloading the map five
// ways over. Two things ride along wider than they look: the view follows a signature
// change because `ghost::reconcile` can raise or drop a node as a side effect of one,
// and the history follows any command because the journal grows without announcing it.

import type { QueryKey } from '@tanstack/svelte-query';

import { key } from '$lib/api/queries';
import type { MapEvent } from '$lib/api/types/MapEvent';

/** `null` is the reconnect catch-up ("you missed something"): the whole map subtree. */
export function keysFor(mapId: number, event: MapEvent | null): QueryKey[] {
	if (event === null) return [key.map(mapId)];
	switch (event.type) {
		case 'characters_changed':
			return [key.mapCharacters(mapId)];
		case 'killmail_received':
			return [key.killmails(mapId)];
		case 'watchlist_changed':
			return [key.watchlist(mapId)];
		case 'ignored_systems_changed':
			return [key.ignored(mapId)];
		// Undo, redo and jumping to a step all publish this, and moving the cursor can
		// touch anything the steps it crosses did — the server says so where it
		// publishes it. Enumerated rather than the whole prefix, so killmails and
		// presence are not dragged along.
		case 'history_changed':
			return [
				key.mapView(mapId),
				key.signatures(mapId),
				key.watchlist(mapId),
				key.ignored(mapId),
				key.history(mapId),
				key.stale(mapId),
				key.details(mapId),
			];
		case 'signature_changed':
			return [key.signatures(mapId), key.mapView(mapId), key.history(mapId)];
		case 'connection_changed':
			return [key.mapView(mapId), key.stale(mapId), key.history(mapId)];
		case 'map_updated':
		case 'access_changed':
			return [key.mapView(mapId)];
		// The notes are a query of their own, and an editor open on them has to see them
		// change, or its next save goes out against a note that is no longer there.
		case 'system_details_changed':
			return [
				key.mapView(mapId),
				key.history(mapId),
				key.systemDetails(mapId, event.map_solar_system_id),
			];
		default:
			return [key.mapView(mapId), key.history(mapId)];
	}
}
