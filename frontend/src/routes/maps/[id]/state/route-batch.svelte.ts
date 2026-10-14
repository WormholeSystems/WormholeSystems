// The routes from the shared origin to every row of a panel. The search itself is the
// planner's shared one, so a panel only reads routes off it. Component init only.

import type { RouteResult } from '$lib/routing/algorithm';
import type { MapState } from './map-state.svelte';

export function routeBatch(
	map: MapState,
	targets: () => number[],
	opts: { extraIgnored?: () => number[] } = {},
) {
	const routes = $derived.by<Map<number, RouteResult>>(() => {
		const origin = map.routeOrigin;
		const wanted = targets();
		if (origin === null || wanted.length === 0) return new Map();
		const search = map.route.search(origin, opts.extraIgnored?.() ?? []);
		return search?.routesTo(wanted) ?? new Map();
	});
	return {
		get routes() {
			return routes;
		},
	};
}
