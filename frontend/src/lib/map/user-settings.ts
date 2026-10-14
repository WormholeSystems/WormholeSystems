// The viewer's own settings on a map: the one way they are written.

import { createMutation, useQueryClient } from '@tanstack/svelte-query';
import { toast } from 'svelte-sonner';

import { api, errorMessage } from '$lib/api/client';
import { key } from '$lib/api/queries';
import type { MapUserSettings } from '$lib/api/types/MapUserSettings';
import type { UpdateMapUserSettings } from '$lib/api/types/UpdateMapUserSettings';
import { MAP_ACTIONS } from '$lib/map/actions';

interface SettingsWrite {
	mapId: number;
	patch: UpdateMapUserSettings;
	/** The caller says how it went itself, so the shared failure toast stays quiet. */
	quiet: boolean;
}

/**
 * Every write of a viewer's settings, wherever it starts: the map, its settings pages or
 * the map list. Component init only (it creates a mutation).
 *
 * The saved settings go straight into the cache rather than being refetched, and a read
 * already on the wire is cancelled first so the slower, older answer cannot win. What a
 * setting changes beyond itself is refreshed here too, so no caller has to remember it.
 */
export function createUserSettingsWriter() {
	const client = useQueryClient();
	const mutation = createMutation(() => ({
		onMutate: ({ mapId }: SettingsWrite) =>
			client.cancelQueries({ queryKey: key.userSettings(mapId) }),
		mutationFn: ({ mapId, patch }: SettingsWrite) => api.updateMapUserSettings(mapId, patch),
		onSuccess: (saved: MapUserSettings, { mapId, patch }: SettingsWrite) => {
			client.setQueryData(key.userSettings(mapId), saved);
			// Sharing your location puts your pilots on the map, or takes them off it.
			if (patch.tracking_allowed !== undefined) {
				void client.invalidateQueries({ queryKey: key.mapCharacters(mapId) });
			}
			// The map list and the top bar's shortcuts read these two.
			if (patch.is_pinned !== undefined || patch.is_archived !== undefined) {
				void client.invalidateQueries({ queryKey: key.maps, exact: true });
			}
		},
		onError: (err: unknown, { quiet }: SettingsWrite) => {
			if (!quiet) toast.error(MAP_ACTIONS.saveSettings.failed, { description: errorMessage(err) });
		},
	}));

	return {
		/** Save and move on; a failure is toasted. */
		save(mapId: number, patch: UpdateMapUserSettings): void {
			mutation.mutate({ mapId, patch, quiet: false });
		},
		/**
		 * Save and wait for the stored settings, for a caller that sequences on them. A
		 * failure is toasted as well as rejected, unless `quiet` says the caller reports it.
		 */
		saveAsync(
			mapId: number,
			patch: UpdateMapUserSettings,
			{ quiet = false }: { quiet?: boolean } = {},
		): Promise<MapUserSettings> {
			return mutation.mutateAsync({ mapId, patch, quiet });
		},
	};
}

export type UserSettingsWriter = ReturnType<typeof createUserSettingsWriter>;
