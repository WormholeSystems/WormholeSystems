// Keeping the `?system=` deep link and the active system in step, for the map screen.

import { afterNavigate, replaceState } from '$app/navigation';
import { page } from '$app/state';

import { deepLinkTarget, systemParamUrl } from './deep-link';
import type { MapState } from './map-state.svelte';

/**
 * Component init only. The link activates its system once the graph is in, exactly once,
 * and from then on the param follows the active system. It is only ever written: clearing
 * it would race the load-time restore, which reads it before the map data has arrived.
 */
export function syncDeepLink(map: MapState): void {
	let applied = false;
	$effect(() => {
		if (applied || !map.loaded) return;
		applied = true;
		if (map.activeId === null) {
			map.activeId = deepLinkTarget(map.systems.all, page.url.searchParams.get('system'));
		}
	});

	// `replaceState` throws until the router has started. The screen is usually mounted
	// after that (the page imports it once hydrated), and then `afterNavigate` has already
	// fired for the first load and will not again; when it is mounted before, the write
	// throws and the next navigation event runs it again.
	let navigations = $state(0);
	afterNavigate(() => navigations++);

	$effect(() => {
		void navigations;
		const active = map.activeSystem;
		if (active?.kind !== 'system') return;
		const url = systemParamUrl(page.url, active.solar_system_id);
		if (!url) return;
		try {
			replaceState(url, {});
		} catch {
			// The router has not started yet; `afterNavigate` brings this back once it has.
		}
	});
}
