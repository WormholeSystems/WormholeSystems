// The ignore-list domain: the systems the whole map routes around and never maps by flying.

import { api } from '$lib/api/client';
import type { IgnoredSystem } from '$lib/api/types/IgnoredSystem';
import type { RunAction } from '$lib/map/actions';

export interface IgnoredHost {
	mapId: number;
	all(): IgnoredSystem[];
	run: RunAction;
}

export class IgnoredApi {
	constructor(private host: IgnoredHost) {}

	get all(): IgnoredSystem[] {
		return this.host.all();
	}

	has(solarSystemId: number): boolean {
		return this.host.all().some((row) => row.solar_system_id === solarSystemId);
	}

	add(solarSystemId: number) {
		this.host.run('ignore', () =>
			api.addIgnored({ map_id: this.host.mapId, solar_system_id: solarSystemId }),
		);
	}

	remove(solarSystemId: number) {
		this.host.run('unignore', () =>
			api.removeIgnored({ map_id: this.host.mapId, solar_system_id: solarSystemId }),
		);
	}

	clear() {
		this.host.run('clearIgnored', () => api.clearIgnored({ map_id: this.host.mapId }));
	}
}
