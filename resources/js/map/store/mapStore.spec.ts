import { createMapStore } from '@/map/store/mapStore';
import { TMap, TMapConnection, TMapSolarsystem, TSolarsystem } from '@/pages/maps';
import { describe, expect, it } from 'vitest';
import { ref } from 'vue';

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

function mapPayload(systems: TMapSolarsystem[], connections: TMapConnection[], overrides: Partial<TMap> = {}): TMap {
    return {
        id: 1,
        name: 'Test',
        slug: 'test',
        home_solarsystem_id: null,
        rally_solarsystem_id: null,
        layout: 'manual',
        allow_layout_override: false,
        constant_width_enabled: false,
        bookmark_format_wormhole: '',
        bookmark_format_kspace: '',
        bookmark_alias_scheme: 'numeric',
        map_solarsystems: systems,
        map_connections: connections,
        ...overrides,
    };
}

describe('entity ops', () => {
    it('upserts a system and mirrors its position into the live position map', () => {
        const store = createMapStore();
        store.upsertSystem(system(1));

        expect(store.systems.get(1)!.id).toBe(1);
        expect(store.positions.get(1)).toEqual({ x: 100, y: 100 });
    });

    it('keeps the live position of a drag-locked system while applying other fields', () => {
        const store = createMapStore();
        store.upsertSystem(system(1));
        store.lockPosition(1);
        store.moveSystem(1, { x: 555, y: 555 });

        store.upsertSystem(system(1, { alias: 'HOME', position: { x: 0, y: 0 } }));

        expect(store.systems.get(1)!.alias).toBe('HOME');
        expect(store.positions.get(1)).toEqual({ x: 555, y: 555 });

        store.unlockPosition(1);
        store.upsertSystem(system(1, { position: { x: 40, y: 40 } }));
        expect(store.positions.get(1)).toEqual({ x: 40, y: 40 });
    });

    it('moveSystem writes only the live position, never the entity', () => {
        const store = createMapStore();
        const entity = system(1);
        store.upsertSystem(entity);
        store.moveSystem(1, { x: 999, y: 999 });

        expect(store.systems.get(1)).toBe(entity);
        expect(store.positions.get(1)).toEqual({ x: 999, y: 999 });
    });

    it('removeSystem drops the entity, its position/size, and its selection membership', () => {
        const store = createMapStore();
        store.upsertSystem(system(1));
        store.nodeSizes.set(1, { width: 180, height: 40 });
        store.setSelection([1]);

        store.removeSystem(1);

        expect(store.systems.has(1)).toBe(false);
        expect(store.positions.has(1)).toBe(false);
        expect(store.nodeSizes.has(1)).toBe(false);
        expect(store.selectedIds.value.has(1)).toBe(false);
    });

    it('patches signature counts by replacing the entity object', () => {
        const store = createMapStore();
        const before = system(1);
        store.upsertSystem(before);
        store.patchSystemCounts(1, { signatures_count: 4, wormhole_signatures_count: 2, uncategorized_signatures_count: 1 });

        const after = store.systems.get(1)!;
        expect(after).not.toBe(before);
        expect(after.signatures_count).toBe(4);
        expect(after.wormhole_signatures_count).toBe(2);
    });
});

describe('reconcileMap', () => {
    it('preserves object identity for unchanged entities and replaces changed ones', () => {
        const store = createMapStore();
        const shared = staticSolarsystem(31000001);
        const a1 = system(1, { solarsystem: shared });
        const b1 = system(2);
        const c1 = connection(10, 1, 2);
        store.reconcileMap(mapPayload([a1, b1], [c1]));

        const keptSystem = store.systems.get(1)!;
        const keptConnection = store.connections.get(10)!;

        // Fresh-but-equal payload objects (new references, same data).
        store.reconcileMap(mapPayload([system(1, { solarsystem: shared }), system(2, { alias: 'CHANGED' }), system(3)], [connection(10, 1, 2)]));

        expect(store.systems.get(1)).toBe(keptSystem);
        expect(store.connections.get(10)).toBe(keptConnection);
        expect(store.systems.get(2)!.alias).toBe('CHANGED');
        expect(store.systems.has(3)).toBe(true);
    });

    it('removes entities missing from the payload and prunes the selection', () => {
        const store = createMapStore();
        store.reconcileMap(mapPayload([system(1), system(2)], [connection(10, 1, 2)]));
        store.setSelection([1, 2]);

        store.reconcileMap(mapPayload([system(1)], []));

        expect(store.systems.has(2)).toBe(false);
        expect(store.connections.has(10)).toBe(false);
        expect(store.selectedIds.value.has(2)).toBe(false);
        expect(store.selectedIds.value.has(1)).toBe(true);
    });

    it('updates the metadata alongside the collections', () => {
        const store = createMapStore();
        store.reconcileMap(mapPayload([], [], { name: 'Renamed', layout: 'tree' }));

        expect(store.meta.value!.name).toBe('Renamed');
        expect(store.effectiveLayout.value).toBe('tree');
    });
});

describe('layout mode', () => {
    it('honours the user override only when the map allows it', () => {
        const store = createMapStore();
        store.reconcileMap(mapPayload([], [], { layout: 'tree', allow_layout_override: false }));
        store.userLayoutOverride.value = 'manual';
        expect(store.effectiveLayout.value).toBe('tree');

        store.reconcileMap(mapPayload([], [], { layout: 'tree', allow_layout_override: true }));
        expect(store.effectiveLayout.value).toBe('manual');
        expect(store.isLayoutLocked.value).toBe(false);
    });

    it('computes tree positions in tree mode and falls back to live positions otherwise', () => {
        const store = createMapStore();
        const home = system(1, { pinned: true });
        store.reconcileMap(mapPayload([home, system(2)], [connection(10, 1, 2)], { layout: 'tree' }));

        expect(store.treePositions.value).not.toBeNull();
        expect(store.renderPosition(1)).toEqual(store.treePositions.value!.get(1));

        store.reconcileMap(mapPayload([home, system(2)], [connection(10, 1, 2)], { layout: 'manual' }));
        expect(store.treePositions.value).toBeNull();
        expect(store.renderPosition(1)).toEqual({ x: 100, y: 100 });
    });
});

describe('derived connection state', () => {
    it('resolves a connection to its endpoint systems', () => {
        const store = createMapStore();
        store.reconcileMap(mapPayload([system(1), system(2)], [connection(10, 1, 2)]));

        const resolved = store.resolveConnection(10)!;
        expect(resolved.source.id).toBe(1);
        expect(resolved.target.id).toBe(2);
        expect(store.resolveConnection(99)).toBeNull();
    });

    it('marks connections whose endpoints are adjacent on the route', () => {
        const path = ref<TSolarsystem[] | null>([staticSolarsystem(31000001), staticSolarsystem(31000002), staticSolarsystem(31000009)]);
        const store = createMapStore({ path });
        store.reconcileMap(mapPayload([system(1), system(2), system(3)], [connection(10, 1, 2), connection(11, 2, 3)]));

        expect(store.routeConnectionIds.value.has(10)).toBe(true);
        expect(store.routeConnectionIds.value.has(11)).toBe(false);

        path.value = null;
        expect(store.routeConnectionIds.value.size).toBe(0);
    });

    it('maps rally-route connections to their animation direction', () => {
        const store = createMapStore({
            getRallyRouteInfo: (from, to) => ({ onRoute: from === 31000001 && to === 31000002, reversed: from > to }),
        });
        store.reconcileMap(mapPayload([system(1), system(2)], [connection(10, 1, 2)]));

        expect(store.rallyEdgeDirections.value.get(10)).toBe('forward');
    });
});

describe('selection helpers', () => {
    it('replaces the selected set wholesale and answers membership', () => {
        const store = createMapStore();
        store.setSelection([1, 2]);
        const first = store.selectedIds.value;
        expect(store.isSelected(1)).toBe(true);

        store.setSelection([2]);
        expect(store.selectedIds.value).not.toBe(first);
        expect(store.isSelected(1)).toBe(false);

        store.clearSelection();
        expect(store.selectedIds.value.size).toBe(0);
    });
});
