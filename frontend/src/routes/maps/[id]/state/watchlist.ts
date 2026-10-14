// The watchlist domain: the systems whose jump distance the navigation panel tracks.

import { api } from '$lib/api/client';
import type { WatchlistEntry } from '$lib/api/types/WatchlistEntry';
import type { RunAction } from '$lib/map/actions';

export interface WatchlistHost {
	mapId: number;
	all(): WatchlistEntry[];
	run: RunAction;
}

export class WatchlistApi {
	constructor(private host: WatchlistHost) {}

	get all(): WatchlistEntry[] {
		return this.host.all();
	}

	add(solarSystemId: number) {
		this.host.run('watch', () =>
			api.addWatchlistEntry({ map_id: this.host.mapId, solar_system_id: solarSystemId }),
		);
	}

	remove(entryId: number) {
		this.host.run('unwatch', () =>
			api.removeWatchlistEntry({ map_id: this.host.mapId, entry_id: entryId }),
		);
	}

	/** Pinned entries surface as route quick-picks. */
	setPinned(entryId: number, value: boolean) {
		this.host.run('setPinned', () =>
			api.setWatchlistPinned({ map_id: this.host.mapId, entry_id: entryId, value }),
		);
	}
}
