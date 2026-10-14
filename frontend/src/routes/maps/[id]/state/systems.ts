// The systems domain: the nodes on the map and every verb that changes them. Components
// read `map.systems.all` and call these; nothing outside `state/` builds api requests.

import { api } from '$lib/api/client';
import type { GridConfig } from '$lib/api/types/GridConfig';
import type { ResolveGhostSystem } from '$lib/api/types/ResolveGhostSystem';
import type { MapSystemView } from '$lib/api/types/MapSystemView';
import type { SystemStatus } from '$lib/api/types/SystemStatus';
import type { RunAction } from '$lib/map/actions';
import { centerWorld, freePosition, type Vec2 } from '$lib/map/geometry';
import { heuristicSize } from '$lib/map/mass';
import type { MapCamera } from './map-camera.svelte';

export interface SystemsHost {
	mapId: number;
	camera: MapCamera;
	all(): MapSystemView[];
	byId(): ReadonlyMap<number, MapSystemView>;
	/** The placements nothing connects back to the rest of the chain. */
	orphaned(): MapSystemView[];
	grid(): GridConfig;
	run: RunAction;
	/** The local echo for a saved note, so it reads back before the server confirms it. */
	notesLocal(mapSolarSystemId: number, notes: string | null): void;
}

export class SystemsApi {
	constructor(private host: SystemsHost) {}

	/** Every node on the map, ghosts included. */
	get all(): MapSystemView[] {
		return this.host.all();
	}

	/** Every node by placement id, built once per graph. */
	get byId(): ReadonlyMap<number, MapSystemView> {
		return this.host.byId();
	}

	/** The dead branches: placements cut off from the rest of the chain. */
	get orphaned(): MapSystemView[] {
		return this.host.orphaned();
	}

	/**
	 * Place a system at a free spot near `anchor` (default: the middle of the view);
	 * `connectFrom` also links the new placement to an existing node in the same action.
	 */
	add(solarSystemId: number, opts: { anchor?: Vec2 | null; connectFrom?: number | null } = {}) {
		const base =
			opts.anchor ??
			centerWorld(this.host.camera.pan, this.host.camera.zoom, this.host.camera.viewportRect());
		const at = freePosition(this.all, base, this.host.grid());
		const from = opts.connectFrom ?? null;
		this.host.run('addSystem', async () => {
			const placed = await api.addSystem({
				map_id: this.host.mapId,
				solar_system_id: solarSystemId,
				x: at.x,
				y: at.y,
				alias: null,
			});
			if (from !== null && from !== placed.id) {
				await api.addConnection({
					map_id: this.host.mapId,
					from_system: from,
					to_system: placed.id,
					kind: 'wormhole',
					size: heuristicSize(this.all, from, placed.id),
				});
			}
		});
	}

	/**
	 * Put every public hole out of a hub (Thera or Turnur) on the map. The server holds the
	 * list; the client only says where the hub goes if it is new: the middle of the view.
	 */
	addEveScout(hubSolarSystemId: number) {
		const at = centerWorld(
			this.host.camera.pan,
			this.host.camera.zoom,
			this.host.camera.viewportRect(),
		);
		this.host.run('addEveScout', () =>
			api.addEveScoutToMap({
				map_id: this.host.mapId,
				hub_solar_system_id: hubSolarSystemId,
				x: at.x,
				y: at.y,
			}),
		);
	}

	/** Move placements to where a drag dropped them; the optimistic override is the caller's. */
	move(moves: { map_solar_system_id: number; x: number; y: number }[]) {
		this.host.run('moveSystems', () => api.moveSystems({ map_id: this.host.mapId, moves }));
	}

	remove(mapSolarSystemIds: number[]) {
		this.host.run('removeSystems', () =>
			api.removeSystems({ map_id: this.host.mapId, map_solar_system_ids: mapSolarSystemIds }),
		);
	}

	/** Take everything off the map. */
	clearAll() {
		this.host.run('clearMap', () => api.clearMap({ map_id: this.host.mapId }));
	}

	/** Take the dead branches off the map. */
	removeOrphaned() {
		const ids = this.orphaned.map((s) => s.id);
		if (ids.length === 0) return;
		this.host.run(
			'cleanMap',
			() => api.removeSystems({ map_id: this.host.mapId, map_solar_system_ids: ids }),
			`${ids.length} ${ids.length === 1 ? 'system' : 'systems'}`,
		);
	}

	/** Say which system a ghost placement turned out to be, and what else the jump learned. */
	assignGhost(cmd: Omit<ResolveGhostSystem, 'map_id'>) {
		this.host.run('assignSystem', () =>
			api.resolveGhostSystem({ ...cmd, map_id: this.host.mapId }),
		);
	}

	/** The alias, and for a real system who holds it; a ghost owns only the alias. */
	rename(system: MapSystemView, alias: string | null, occupier: string | null) {
		const target = { map_id: this.host.mapId, map_solar_system_id: system.id };
		this.host.run('setAlias', () =>
			Promise.all([
				api.setAlias({ ...target, alias }),
				system.kind === 'system' ? api.setOccupier({ ...target, occupier }) : null,
			]),
		);
	}

	saveNotes(mapSolarSystemId: number, notes: string | null) {
		this.host.run('setNotes', () =>
			api.setNotes({ map_id: this.host.mapId, map_solar_system_id: mapSolarSystemId, notes }),
		);
		this.host.notesLocal(mapSolarSystemId, notes);
	}

	setStatus(mapSolarSystemId: number, status: SystemStatus) {
		this.host.run('setStatus', () =>
			api.setStatus({ map_id: this.host.mapId, map_solar_system_id: mapSolarSystemId, status }),
		);
	}

	setPinned(mapSolarSystemId: number, value: boolean) {
		this.host.run('setPinned', () =>
			api.setPinned({ map_id: this.host.mapId, map_solar_system_id: mapSolarSystemId, value }),
		);
	}

	setHome(mapSolarSystemId: number, value: boolean) {
		this.host.run('setHome', () =>
			api.setHome({ map_id: this.host.mapId, map_solar_system_id: mapSolarSystemId, value }),
		);
	}

	setRally(mapSolarSystemId: number, value: boolean) {
		this.host.run('setRally', () =>
			api.setRally({ map_id: this.host.mapId, map_solar_system_id: mapSolarSystemId, value }),
		);
	}
}
