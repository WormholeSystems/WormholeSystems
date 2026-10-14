// The placement domain: how the chain is laid out, for everyone on the map or for one
// viewer on top of that.

import { api } from '$lib/api/client';
import type { MapLayout } from '$lib/api/types/MapLayout';
import type { MapUserSettings } from '$lib/api/types/MapUserSettings';
import type { UpdateMapUserSettings } from '$lib/api/types/UpdateMapUserSettings';
import type { RunAction } from '$lib/map/actions';

export interface PlacementHost {
	mapId: number;
	/** The map's own mode, null until the map has loaded. */
	mapLayout(): MapLayout | null;
	run: RunAction;
	patchSettingsLocal(update: (s: MapUserSettings) => MapUserSettings): void;
	patchUserSettings(patch: UpdateMapUserSettings): void;
}

export class PlacementApi {
	constructor(private host: PlacementHost) {}

	/** The map's own mode, which everyone without an override of their own follows. */
	setForMap(layout: MapLayout) {
		this.host.run('setPlacement', () => api.updateMap({ map_id: this.host.mapId, layout }));
	}

	/**
	 * Pick a placement for yourself. Choosing the map's own mode clears the override
	 * rather than pinning the same value, so a later change to the map still reaches you.
	 */
	setForMe(mode: MapLayout) {
		const own = mode === this.host.mapLayout() ? null : mode;
		this.host.patchSettingsLocal((s) => ({ ...s, layout_override: own ?? undefined }));
		this.host.patchUserSettings({ layout_override: own });
	}
}
