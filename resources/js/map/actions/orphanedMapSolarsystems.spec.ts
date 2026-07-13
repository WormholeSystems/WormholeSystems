import { getClearableMapSolarsystems } from '@/map/actions/clearableMapSolarsystems';
import { getOrphanedMapSolarsystems } from '@/map/actions/orphanedMapSolarsystems';
import { createMapStore } from '@/map/store/mapStore';
import { TMap, TMapConnection, TMapSolarsystem, TSolarsystem } from '@/pages/maps';
import { describe, expect, it } from 'vitest';

const staticSolarsystem = (id: number, name = `J${id}`): TSolarsystem =>
    ({
        id,
        name,
        class: '3',
        region: { id: 1, name: 'A-R00001' },
    }) as TSolarsystem;

function system(id: number, overrides: Partial<TMapSolarsystem> = {}): TMapSolarsystem {
    return {
        id,
        map_id: 1,
        solarsystem_id: 31000000 + id,
        alias: null,
        status: 'unknown',
        occupier_alias: null,
        notes: null,
        position: { x: 100 * id, y: 100 },
        pinned: false,
        signatures_count: 0,
        wormhole_signatures_count: 0,
        map_connections_count: 0,
        uncategorized_signatures_count: 0,
        solarsystem: staticSolarsystem(31000000 + id),
        ...overrides,
    } as TMapSolarsystem;
}

function connection(id: number, from: number, to: number, overrides: Partial<TMapConnection> = {}): TMapConnection {
    return {
        id,
        from_map_solarsystem_id: from,
        to_map_solarsystem_id: to,
        type: 'wormhole',
        preserve_mass: false,
        mass_status: 'fresh',
        lifetime_status: 'healthy',
        lifetime_status_updated_at: null,
        signatures: [],
        ship_size: 'large',
        created_at: '2026-01-01T00:00:00Z',
        updated_at: '2026-01-01T00:00:00Z',
        ...overrides,
    } as TMapConnection;
}

function storeWith(systems: TMapSolarsystem[], connections: TMapConnection[], home_solarsystem_id: number | null = null) {
    const store = createMapStore();
    store.reconcileMap({
        id: 1,
        name: 'Test',
        slug: 'test',
        home_solarsystem_id,
        rally_solarsystem_id: null,
        layout: 'manual',
        allow_layout_override: false,
        constant_width_enabled: false,
        bookmark_format_wormhole: '',
        bookmark_format_kspace: '',
        map_solarsystems: systems,
        map_connections: connections,
    } as TMap);
    return store;
}

describe('getClearableMapSolarsystems', () => {
    it('excludes pinned systems and the home system', () => {
        const store = storeWith([system(1, { pinned: true }), system(2), system(3)], [], 31000003);

        const clearable = getClearableMapSolarsystems(store).map((s) => s.id);

        expect(clearable).toEqual([2]);
    });
});

describe('getOrphanedMapSolarsystems', () => {
    it('keeps systems reachable from a pinned or home anchor and drops disconnected branches', () => {
        // 1 (home) — 2 — 3 chain, 4 (pinned) — 5 branch, 6 — 7 dead branch, 8 isolated.
        const store = storeWith(
            [system(1), system(2), system(3), system(4, { pinned: true }), system(5), system(6), system(7), system(8)],
            [connection(10, 1, 2), connection(11, 2, 3), connection(12, 4, 5), connection(13, 6, 7)],
            31000001,
        );

        const orphaned = getOrphanedMapSolarsystems(store)
            .map((s) => s.id)
            .sort();

        expect(orphaned).toEqual([6, 7, 8]);
    });

    it('traverses connections in both directions', () => {
        const store = storeWith([system(1, { pinned: true }), system(2)], [connection(10, 2, 1)]);

        expect(getOrphanedMapSolarsystems(store)).toEqual([]);
    });

    it('treats every system as orphaned when the map has no anchors', () => {
        const store = storeWith([system(1), system(2)], [connection(10, 1, 2)]);

        const orphaned = getOrphanedMapSolarsystems(store)
            .map((s) => s.id)
            .sort();

        expect(orphaned).toEqual([1, 2]);
    });
});
