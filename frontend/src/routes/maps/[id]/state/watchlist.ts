// The watchlist domain: the systems whose jump distance the navigation panel tracks.

import { api } from '$lib/api/client';
import type { WatchlistEntry } from '$lib/api/types/WatchlistEntry';
import type { RunAction } from '$lib/map/actions';

export interface WatchlistHost {
	mapId: number;
	all(): WatchlistEntry[];
	run: RunAction;
}

/** What the watchlist rows need, from either list. */
export interface WatchlistList {
	readonly all: WatchlistEntry[];
	add(solarSystemId: number): void;
	/** Settles with the write, so a row can show it is on its way out. */
	remove(entryId: number): Promise<unknown>;
	setPinned(entryId: number, value: boolean): Promise<unknown>;
}

export class WatchlistApi implements WatchlistList {
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
		return this.host.run('unwatch', () =>
			api.removeWatchlistEntry({ map_id: this.host.mapId, entry_id: entryId }),
		);
	}

	/** Pinned entries surface as route quick-picks. */
	setPinned(entryId: number, value: boolean) {
		return this.host.run('setPinned', () =>
			api.setWatchlistPinned({ map_id: this.host.mapId, entry_id: entryId, value }),
		);
	}
}

export interface PersonalWatchlistHost {
	mapId: number;
	all(): WatchlistEntry[];
	run: RunAction;
}

/**
 * The user's own watchlist on this map: the same verbs as the shared one, but nobody else
 * sees it and it is not a step in the map's history.
 */
export class PersonalWatchlistApi implements WatchlistList {
	constructor(private host: PersonalWatchlistHost) {}

	get all(): WatchlistEntry[] {
		return this.host.all();
	}

	add(solarSystemId: number) {
		void this.write('watch', () =>
			api.addPersonalWatchlistEntry(this.host.mapId, { solar_system_id: solarSystemId }),
		);
	}

	remove(entryId: number) {
		return this.write('unwatch', () =>
			api.removePersonalWatchlistEntry(this.host.mapId, { entry_id: entryId }),
		);
	}

	setPinned(entryId: number, value: boolean) {
		return this.write('setPinned', () =>
			api.setPersonalWatchlistPinned(this.host.mapId, { entry_id: entryId, value }),
		);
	}

	// No refetch here: the user's own channel echoes every write to each of their tabs,
	// this one included, and that is what refreshes the list.
	private write(action: 'watch' | 'unwatch' | 'setPinned', exec: () => Promise<unknown>) {
		return this.host.run(action, exec);
	}
}
