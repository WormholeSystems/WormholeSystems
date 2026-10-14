// The map behind the system menus. They are wrapped around rows deep inside the panels, so
// the map screen hands its state down through context rather than through every row.

import { getContext, setContext } from 'svelte';

import type { MapState } from '../../state/map-state.svelte';

const KEY = 'map-state';

/** Installed by the map screen; the getter keeps consumers on the live instance. */
export function setMapContext(get: () => MapState): void {
	setContext(KEY, get);
}

/** The map the menu belongs to. Only the map screen renders these menus. */
export function getMapContext(): () => MapState {
	return getContext<() => MapState>(KEY);
}
