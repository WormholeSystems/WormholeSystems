import {
    CharacterStatusUpdatedEvent,
    MapConnectionsRemovedEvent,
    MapConnectionsUpsertedEvent,
    MapIgnoredSolarsystemsUpdatedEvent,
    MapMetadataUpdatedEvent,
    MapResyncEvent,
    MapRouteSolarsystemsUpdatedEvent,
    MapSolarsystemsRemovedEvent,
    MapSolarsystemsUpsertedEvent,
    SignaturesChangedEvent,
} from '@/const/events';
import type { SystemCounts } from '@/map/store/entities';
import type { TMapMeta } from '@/map/store/derived';
import { TMapConnection, TMapSolarsystem, TResolvedSolarsystem, TServerMapSolarsystem } from '@/pages/maps';

/**
 * The store surface the sync handlers write to — kept narrow so the handler
 * table stays unit-testable against a plain fake.
 */
export type SyncStore = {
    upsertSystem(system: TMapSolarsystem): void;
    removeSystem(id: number): void;
    upsertConnection(connection: TMapConnection): void;
    removeConnection(id: number): void;
    patchSystemCounts(id: number, counts: SystemCounts): void;
    patchMapMeta(meta: TMapMeta): void;
};

/**
 * What should happen besides the store writes: `reload` names per-user Inertia
 * props to refetch (coalesced); `reloadIfSelected` adds the detail panel prop
 * only when the currently selected system is among the given ids.
 */
export type SyncEffect = {
    reload?: string[];
    reloadIfSelected?: number[];
};

export type ResolveSolarsystem = (solarsystemId: number) => TResolvedSolarsystem;

type SystemsUpsertedPayload = { map_solarsystems: TServerMapSolarsystem[] };
type SystemsRemovedPayload = { map_solarsystem_ids: number[]; map_connection_ids: number[] };
type ConnectionsUpsertedPayload = { map_connections: TMapConnection[] };
type ConnectionsRemovedPayload = { map_connection_ids: number[]; map_solarsystem_ids: number[] };
type MetadataPayload = { map: TMapMeta };
type SignaturesChangedPayload = { map_solarsystem_id: number; signature_counts: SystemCounts };

export type SyncHandler = (store: SyncStore, payload: unknown, resolveSolarsystem: ResolveSolarsystem) => SyncEffect;

/**
 * The entire realtime sync contract in one table: broadcast event → store
 * operations + follow-up prop reloads. Payload events patch shared map state
 * directly; per-user derived props stay ping + coalesced partial reload.
 */
export const mapEventHandlers: Record<string, SyncHandler> = {
    [MapSolarsystemsUpsertedEvent]: (store, payload, resolveSolarsystem) => {
        const { map_solarsystems } = payload as SystemsUpsertedPayload;
        for (const system of map_solarsystems) {
            store.upsertSystem({ ...system, solarsystem: resolveSolarsystem(system.solarsystem_id) });
        }
        return { reload: ['map_navigation'], reloadIfSelected: map_solarsystems.map((system) => system.id) };
    },

    [MapSolarsystemsRemovedEvent]: (store, payload) => {
        const { map_solarsystem_ids, map_connection_ids } = payload as SystemsRemovedPayload;
        map_connection_ids.forEach((id) => store.removeConnection(id));
        map_solarsystem_ids.forEach((id) => store.removeSystem(id));
        return { reload: ['map_navigation'], reloadIfSelected: map_solarsystem_ids };
    },

    [MapConnectionsUpsertedEvent]: (store, payload) => {
        const { map_connections } = payload as ConnectionsUpsertedPayload;
        map_connections.forEach((connection) => store.upsertConnection(connection));
        return {
            reload: ['map_navigation', 'eve_scout_connections'],
            reloadIfSelected: map_connections.flatMap((connection) => [connection.from_map_solarsystem_id, connection.to_map_solarsystem_id]),
        };
    },

    [MapConnectionsRemovedEvent]: (store, payload) => {
        const { map_connection_ids, map_solarsystem_ids } = payload as ConnectionsRemovedPayload;
        map_connection_ids.forEach((id) => store.removeConnection(id));
        map_solarsystem_ids.forEach((id) => store.removeSystem(id));
        return { reload: ['map_navigation', 'eve_scout_connections'], reloadIfSelected: map_solarsystem_ids };
    },

    [MapMetadataUpdatedEvent]: (store, payload) => {
        store.patchMapMeta((payload as MetadataPayload).map);
        return {};
    },

    [SignaturesChangedEvent]: (store, payload) => {
        const { map_solarsystem_id, signature_counts } = payload as SignaturesChangedPayload;
        store.patchSystemCounts(map_solarsystem_id, signature_counts);
        return { reloadIfSelected: [map_solarsystem_id] };
    },

    /** Batch too large for a payload event — one coalesced full-map refetch instead. */
    [MapResyncEvent]: () => ({ reload: ['map'] }),

    // Per-user / derived props: contentless pings, coalesced partial reloads.
    [MapRouteSolarsystemsUpdatedEvent]: () => ({ reload: ['map_navigation'] }),
    [MapIgnoredSolarsystemsUpdatedEvent]: () => ({ reload: ['map_ignored_systems'] }),
    [CharacterStatusUpdatedEvent]: () => ({ reload: ['map_characters', 'ship_history'] }),
};

/** Every prop the map page hydrates from — used as the post-reconnect resync set. */
export const ALL_MAP_PROPS = [
    'map',
    'selected_map_solarsystem',
    'map_characters',
    'map_navigation',
    'eve_scout_connections',
    'map_killmails',
    'ship_history',
    'map_ignored_systems',
];
