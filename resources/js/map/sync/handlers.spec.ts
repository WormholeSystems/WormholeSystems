import {
    MapConnectionsRemovedEvent,
    MapConnectionsUpsertedEvent,
    MapMetadataUpdatedEvent,
    MapResyncEvent,
    MapRouteSolarsystemsUpdatedEvent,
    MapSolarsystemsRemovedEvent,
    MapSolarsystemsUpsertedEvent,
    SignaturesChangedEvent,
} from '@/const/events';
import { mapEventHandlers, type ResolveSolarsystem, type SyncStore } from '@/map/sync/handlers';
import { createReloadCoalescer } from '@/map/sync/reloadCoalescer';
import { TResolvedSolarsystem } from '@/pages/maps';
import { describe, expect, it, vi } from 'vitest';

function fakeStore() {
    const upserted: unknown[] = [];
    const removedSystems: number[] = [];
    const upsertedConnections: unknown[] = [];
    const removedConnections: number[] = [];
    const countPatches: unknown[] = [];
    const metaPatches: unknown[] = [];

    const store: SyncStore = {
        upsertSystem: (system) => void upserted.push(system),
        removeSystem: (id) => void removedSystems.push(id),
        upsertConnection: (connection) => void upsertedConnections.push(connection),
        removeConnection: (id) => void removedConnections.push(id),
        patchSystemCounts: (id, counts) => void countPatches.push({ id, counts }),
        patchMapMeta: (meta) => void metaPatches.push(meta),
    };

    return { ...store, upserted, removedSystems, upsertedConnections, removedConnections, countPatches, metaPatches };
}

const resolve: ResolveSolarsystem = (id) => ({ id, name: `J${id}` }) as TResolvedSolarsystem;

describe('mapEventHandlers', () => {
    it('upserts systems with the static solarsystem resolved and flags the detail panel for them', () => {
        const store = fakeStore();
        const effect = mapEventHandlers[MapSolarsystemsUpsertedEvent](
            store,
            { map_solarsystems: [{ id: 7, solarsystem_id: 31000001 }] },
            resolve,
        );

        expect(store.upserted).toHaveLength(1);
        expect((store.upserted[0] as { solarsystem: { name: string } }).solarsystem.name).toBe('J31000001');
        expect(effect.reload).toEqual(['map_navigation']);
        expect(effect.reloadIfSelected).toEqual([7]);
    });

    it('removes systems and their cascaded connections', () => {
        const store = fakeStore();
        const effect = mapEventHandlers[MapSolarsystemsRemovedEvent](
            store,
            { map_solarsystem_ids: [1, 2], map_connection_ids: [10] },
            resolve,
        );

        expect(store.removedSystems).toEqual([1, 2]);
        expect(store.removedConnections).toEqual([10]);
        expect(effect.reloadIfSelected).toEqual([1, 2]);
    });

    it('upserts connections and flags both endpoints for the detail panel', () => {
        const store = fakeStore();
        const effect = mapEventHandlers[MapConnectionsUpsertedEvent](
            store,
            { map_connections: [{ id: 10, from_map_solarsystem_id: 1, to_map_solarsystem_id: 2 }] },
            resolve,
        );

        expect(store.upsertedConnections).toHaveLength(1);
        expect(effect.reload).toEqual(['map_navigation', 'eve_scout_connections']);
        expect(effect.reloadIfSelected).toEqual([1, 2]);
    });

    it('removes connections plus cascade-removed systems', () => {
        const store = fakeStore();
        mapEventHandlers[MapConnectionsRemovedEvent](store, { map_connection_ids: [10, 11], map_solarsystem_ids: [3] }, resolve);

        expect(store.removedConnections).toEqual([10, 11]);
        expect(store.removedSystems).toEqual([3]);
    });

    it('patches metadata and signature counts in place', () => {
        const store = fakeStore();
        mapEventHandlers[MapMetadataUpdatedEvent](store, { map: { id: 1, name: 'Renamed' } }, resolve);
        const countsEffect = mapEventHandlers[SignaturesChangedEvent](
            store,
            { map_solarsystem_id: 5, signature_counts: { signatures_count: 3, wormhole_signatures_count: 1, uncategorized_signatures_count: 0 } },
            resolve,
        );

        expect(store.metaPatches).toHaveLength(1);
        expect(store.countPatches).toEqual([{ id: 5, counts: { signatures_count: 3, wormhole_signatures_count: 1, uncategorized_signatures_count: 0 } }]);
        expect(countsEffect.reloadIfSelected).toEqual([5]);
    });

    it('maps resync and ping events to pure reload effects', () => {
        const store = fakeStore();
        expect(mapEventHandlers[MapResyncEvent](store, {}, resolve)).toEqual({ reload: ['map'] });
        expect(mapEventHandlers[MapRouteSolarsystemsUpdatedEvent](store, {}, resolve)).toEqual({ reload: ['map_navigation'] });
        expect(store.upserted).toHaveLength(0);
    });
});

describe('createReloadCoalescer', () => {
    it('unions prop names from a burst into a single reload', () => {
        vi.useFakeTimers();
        const reload = vi.fn();
        const coalescer = createReloadCoalescer({ reload, delayMs: 150 });

        coalescer.schedule(['map_navigation']);
        coalescer.schedule(['map_navigation', 'eve_scout_connections']);
        coalescer.schedule(['selected_map_solarsystem']);

        expect(reload).not.toHaveBeenCalled();
        vi.advanceTimersByTime(150);
        expect(reload).toHaveBeenCalledTimes(1);
        expect(reload).toHaveBeenCalledWith(['map_navigation', 'eve_scout_connections', 'selected_map_solarsystem']);
        vi.useRealTimers();
    });

    it('ignores empty schedules and clears after a flush', () => {
        vi.useFakeTimers();
        const reload = vi.fn();
        const coalescer = createReloadCoalescer({ reload });

        coalescer.schedule([]);
        vi.advanceTimersByTime(500);
        expect(reload).not.toHaveBeenCalled();

        coalescer.schedule(['map']);
        vi.advanceTimersByTime(150);
        coalescer.flushNow();
        expect(reload).toHaveBeenCalledTimes(1);
        vi.useRealTimers();
    });
});
