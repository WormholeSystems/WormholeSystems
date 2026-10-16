// One visit's plumbing: both sockets and the routing tables. Returns the cleanup, so a
// single $effect can hand the whole lifecycle over. The data itself lives in queries,
// which carry their own polling and focus refetching.

import { openMapSocket, openUserSocket } from '$lib/ws';
import type { MapState } from './map-state.svelte';

export function connectMapSession(map: MapState): (() => void) | undefined {
	if (map.mapId === 0) return;
	map.route.load();
	map.route.loadIgnored();
	// A frame names what changed; `null` is the reconnect catch-up, "reload everything".
	const closeWs = openMapSocket(
		map.mapId,
		(event) => map.applyEvent(event),
		(state) => (map.socket = state),
	);
	// Below here is about the pilot at the keyboard: jump tracking and the private
	// channel. A watcher has none of it.
	if (!map.signedIn) return () => closeWs();
	map.characters.refreshMine();
	// The character's own status change is how a jump is normally noticed within seconds.
	const closeUserWs = openUserSocket(
		(event) => {
			if (event.type === 'character_status_changed') map.characters.refreshMine();
		},
		// Back after a gap (a deploy, a dropped connection): a jump may have gone unheard.
		() => map.characters.refreshMine(),
	);
	// Flying happens in the game client, so a jump has usually already happened by the
	// time the tab is looked at again. An explicit listener, not the query's own focus
	// refetching: that watches visibilitychange, and this is about window focus.
	const observe = () => map.characters.refreshMine();
	window.addEventListener('focus', observe);
	return () => {
		window.removeEventListener('focus', observe);
		closeUserWs();
		closeWs();
	};
}
