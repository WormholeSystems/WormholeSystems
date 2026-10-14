// What every settings page needs about the map, set up once by the section's layout: which
// map, the live view and the viewer's role on it, and the viewer's own settings.

import { createQuery, useQueryClient } from '@tanstack/svelte-query';
import { getContext, setContext } from 'svelte';

import { key, q } from '$lib/api/queries';
import type { MapUserSettings } from '$lib/api/types/MapUserSettings';
import type { MapView } from '$lib/api/types/MapView';
import type { UpdateMapUserSettings } from '$lib/api/types/UpdateMapUserSettings';
import { atLeast } from '$lib/map/roles';
import { createUserSettingsWriter } from '$lib/map/user-settings';

export interface MapSettings {
	readonly mapId: number;
	/** The map as the cache has it now, so a save or someone else's edit shows at once. */
	readonly view: MapView;
	/** Read off the live view, so a role changed while the page is open takes effect. */
	readonly canManage: boolean;
	/** The viewer's own settings, null until they arrive. */
	readonly settings: MapUserSettings | null;
	/** Save some of the viewer's own settings; a failure is toasted. */
	saveUserSettings(patch: UpdateMapUserSettings): void;
}

const KEY = Symbol('map-settings');

/**
 * Component init only, in the settings layout. The load's view seeds the cache with
 * `setQueryData` rather than `initialData`, which is ignored once the entry exists and would
 * throw away what this navigation just fetched; until the seed lands, the load's own copy
 * stands in.
 */
export function provideMapSettings(mapId: () => number, loaded: () => MapView): MapSettings {
	const client = useQueryClient();
	$effect.pre(() => {
		client.setQueryData(key.mapView(mapId()), loaded());
	});
	const viewQuery = createQuery(() => q.mapView(mapId()));
	const settingsQuery = createQuery(() => q.mapUserSettings(mapId()));
	const writer = createUserSettingsWriter();

	const view = $derived(viewQuery.data ?? loaded());
	const canManage = $derived(atLeast(view.role, 'manager'));

	const context: MapSettings = {
		get mapId() {
			return mapId();
		},
		get view() {
			return view;
		},
		get canManage() {
			return canManage;
		},
		get settings() {
			return settingsQuery.data ?? null;
		},
		saveUserSettings(patch) {
			writer.save(mapId(), patch);
		},
	};
	setContext(KEY, context);
	return context;
}

/** The section's shared context, for a page under the settings layout. */
export function mapSettings(): MapSettings {
	return getContext<MapSettings>(KEY);
}
