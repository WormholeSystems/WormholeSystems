// The connections domain: the edges of the chain, their degradable properties, and the
// jump log kept per wormhole.

import { api } from '$lib/api/client';
import type { AddConnectionJump } from '$lib/api/types/AddConnectionJump';
import type { MapConnection } from '$lib/api/types/MapConnection';
import type { MapSystemView } from '$lib/api/types/MapSystemView';
import type { SetConnectionStatus } from '$lib/api/types/SetConnectionStatus';
import type { StaleConnection } from '$lib/api/types/StaleConnection';
import type { TrackJump } from '$lib/api/types/TrackJump';
import type { UpdateConnectionJump } from '$lib/api/types/UpdateConnectionJump';
import type { MapAction } from '$lib/map/actions';
import { heuristicSize } from '$lib/map/mass';

export type ConnectionPatch = Omit<SetConnectionStatus, 'map_id' | 'connection_id'>;

// Each degradable property keeps its own action copy, so the toast still says what
// actually changed.
const ACTION_FOR: Record<string, MapAction> = {
	kind: 'setConnectionType',
	mass_status: 'setConnectionMass',
	time_status: 'setConnectionLifetime',
	size: 'setConnectionSize',
	preserve_mass: 'setPreserveMass',
};

export interface ConnectionsHost {
	mapId: number;
	all(): MapConnection[];
	stale(): StaleConnection[];
	systems(): MapSystemView[];
	run(action: MapAction, promise: Promise<unknown>, detail?: string): void;
	/** Ask for a fresh jump log for one connection; a closed popover just goes stale. */
	refreshJumps(connectionId: number): void;
}

export class ConnectionsApi {
	constructor(private host: ConnectionsHost) {}

	get all(): MapConnection[] {
		return this.host.all();
	}

	/** Connections critical for over an hour, offered for a one-click sweep. */
	get stale(): StaleConnection[] {
		return this.host.stale();
	}

	/** Join two placements with a wormhole, sized by what the two systems suggest. */
	add(from: number, to: number) {
		this.host.run(
			'addConnection',
			api.addConnection({
				map_id: this.host.mapId,
				from_system: from,
				to_system: to,
				kind: 'wormhole',
				size: heuristicSize(this.host.systems(), from, to),
			}),
		);
	}

	remove(connectionId: number) {
		this.host.run(
			'removeConnection',
			api.removeConnection({ map_id: this.host.mapId, connection_id: connectionId }),
		);
	}

	/** One write path for the degradable properties, wherever the edit comes from. */
	patch(connectionId: number, patch: ConnectionPatch) {
		const named = Object.keys(patch).find((k) => k in ACTION_FOR);
		this.host.run(
			ACTION_FOR[named ?? 'kind'],
			api.setConnectionStatus({
				map_id: this.host.mapId,
				connection_id: connectionId,
				...patch,
			}),
		);
	}

	cleanStale() {
		this.host.run('cleanStale', api.cleanStaleConnections({ map_id: this.host.mapId }));
	}

	/** Map an observed jump: the tracker's create-or-reuse of a connection plus its log line. */
	trackJump(cmd: Omit<TrackJump, 'map_id'>) {
		this.host.run('trackJump', api.trackJump({ ...cmd, map_id: this.host.mapId }));
	}

	addJump(cmd: Omit<AddConnectionJump, 'map_id'>): Promise<unknown> {
		const write = api.addConnectionJump({ ...cmd, map_id: this.host.mapId });
		this.host.run('addJump', write);
		return write;
	}

	updateJump(cmd: Omit<UpdateConnectionJump, 'map_id'>): Promise<unknown> {
		const write = api.updateConnectionJump({ ...cmd, map_id: this.host.mapId });
		this.host.run('updateJump', write);
		return write;
	}

	removeJump(jumpPk: number): Promise<unknown> {
		const write = api.removeConnectionJump({ map_id: this.host.mapId, jump_pk: jumpPk });
		this.host.run('removeJump', write);
		return write;
	}

	refreshJumps(connectionId: number) {
		this.host.refreshJumps(connectionId);
	}
}
