import type { MapSystemView } from '$lib/api/types/MapSystemView';
import type { SystemSearchResult } from '$lib/api/types/SystemSearchResult';

/**
 * The mapped half of a map node (the other is a ghost), named, so a function can say it
 * takes one instead of accepting either and checking again inside. Narrowed out of the wire
 * union rather than restated, so it cannot drift from what the server sends.
 */
export type MappedSystem = Extract<MapSystemView, { kind: 'system' }>;

/** For the callers that only want the id and do not care which half they were handed. */
export function solarSystemId(node: MapSystemView): number | null {
	return node.kind === 'system' ? node.solar_system_id : null;
}

export function systemName(node: MapSystemView): string | null {
	return node.kind === 'system' ? node.name : null;
}

/**
 * A `SystemSearchResult` from a payload that carries only part of one, with the gaps
 * spelled out: rows built from kills or skyhooks wrap a `SystemMenu` around themselves
 * without waiting on a resolver round trip.
 */
export function toSearchResult(
	base: Pick<SystemSearchResult, 'id' | 'name' | 'security' | 'region'> &
		Partial<SystemSearchResult>,
): SystemSearchResult {
	return {
		region_id: 0,
		constellation_id: 0,
		wormhole_class_id: null,
		effect_name: null,
		sovereignty: null,
		statics: [],
		...base,
	};
}
