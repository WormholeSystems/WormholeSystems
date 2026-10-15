import { describe, expect, it } from 'vitest';

import { key } from '$lib/api/queries';
import type { MapEvent } from '$lib/api/types/MapEvent';
import { keysFor } from './invalidations';

const event = (type: MapEvent['type']): MapEvent => ({ type, map_id: 1 }) as MapEvent;

describe('keysFor', () => {
	it('invalidates the whole map subtree on the reconnect catch-up', () => {
		expect(keysFor(1, null)).toEqual([key.map(1)]);
	});

	it('routes presence to the characters key alone', () => {
		expect(keysFor(1, event('characters_changed'))).toEqual([key.mapCharacters(1)]);
	});

	it('routes a kill to the killmails prefix alone', () => {
		expect(keysFor(1, event('killmail_received'))).toEqual([key.killmails(1)]);
	});

	it('routes a watchlist change to the watchlist alone', () => {
		expect(keysFor(1, event('watchlist_changed'))).toEqual([key.watchlist(1)]);
	});

	it('routes an ignore-list change to the ignore list alone', () => {
		expect(keysFor(1, event('ignored_systems_changed'))).toEqual([key.ignored(1)]);
	});

	it('routes a history change to every slice, but not killmails or presence', () => {
		expect(keysFor(1, event('history_changed'))).toEqual([
			key.mapView(1),
			key.signatures(1),
			key.watchlist(1),
			key.ignored(1),
			key.history(1),
			key.stale(1),
			key.details(1),
		]);
	});

	it('drags the view along with a signature change, for ghost reconciliation', () => {
		expect(keysFor(1, event('signature_changed'))).toEqual([
			key.signatures(1),
			key.mapView(1),
			key.history(1),
		]);
	});

	it('drags the stale list along with a connection change', () => {
		expect(keysFor(1, event('connection_changed'))).toEqual([
			key.mapView(1),
			key.stale(1),
			key.history(1),
		]);
	});

	it('routes map and access changes to the view alone', () => {
		expect(keysFor(1, event('map_updated'))).toEqual([key.mapView(1)]);
		expect(keysFor(1, event('access_changed'))).toEqual([key.mapView(1)]);
	});

	it('routes every structural change to view and history', () => {
		for (const type of ['system_added', 'system_moved', 'system_removed'] as const) {
			expect(keysFor(1, event(type))).toEqual([key.mapView(1), key.history(1)]);
		}
	});

	it('refreshes the one placement whose notes changed, for an editor open on them', () => {
		const changed: MapEvent = { type: 'system_details_changed', map_id: 1, map_solar_system_id: 7 };
		expect(keysFor(1, changed)).toEqual([key.mapView(1), key.history(1), key.systemDetails(1, 7)]);
	});
});
