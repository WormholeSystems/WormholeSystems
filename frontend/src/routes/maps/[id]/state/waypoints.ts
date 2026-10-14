// The waypoints domain: setting in-game destinations from any menu. These land in the
// EVE client, not on the map, which is why the action copy speaks up on success.

import { api } from '$lib/api/client';
import type { RunAction } from '$lib/map/actions';

export interface WaypointsHost {
	run: RunAction;
}

export class WaypointsApi {
	constructor(private host: WaypointsHost) {}

	set(destinationId: number, characterId: number, clearOthers: boolean) {
		this.host.run('setWaypoint', () =>
			api.setWaypoint({
				character_id: characterId,
				destination_id: destinationId,
				clear_other_waypoints: clearOthers,
			}),
		);
	}

	setAll(destinationId: number, clearOthers: boolean) {
		this.host.run('setWaypoint', () =>
			api.setWaypointAll({ destination_id: destinationId, clear_other_waypoints: clearOthers }),
		);
	}
}
