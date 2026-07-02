import { computeTreeLayout, type TreeLayoutInput } from '@/map/core/layout/treeLayout';
import { compareSystems } from '@/map/core/sorting';
import type { Vec2 } from '@/map/core/types';
import { TMap, TMapConnection, TMapSolarsystem, TSolarsystem } from '@/pages/maps';
import { computed, type ComputedRef, type Ref, type ShallowRef } from 'vue';
import type { EntityState } from './entities';
import type { ViewState } from './viewState';

/** Map metadata without the entity collections (those live in the keyed Maps). */
export type TMapMeta = Omit<TMap, 'map_solarsystems' | 'map_connections'>;

export type TResolvedConnection = {
    connection: TMapConnection;
    source: TMapSolarsystem;
    target: TMapSolarsystem;
};

/**
 * External route state the store derives edge highlighting from. Injected by the
 * component root (where the routing composables are available) and optional so
 * the store stays testable without the routing worker.
 */
export type RouteDeps = {
    path?: Readonly<Ref<readonly TSolarsystem[] | null>>;
    getRallyRouteInfo?: (fromSolarsystemId: number, toSolarsystemId: number) => { onRoute: boolean; reversed: boolean };
};

export type DerivedState = ReturnType<typeof createDerivedState>;

export function createDerivedState(entities: EntityState, view: ViewState, meta: ShallowRef<TMapMeta | null>, routeDeps: RouteDeps = {}) {
    const effectiveLayout = computed<'manual' | 'tree'>(() => {
        const metaValue = meta.value;
        if (!metaValue) return 'manual';
        if (metaValue.allow_layout_override && view.userLayoutOverride.value) {
            return view.userLayoutOverride.value;
        }
        return metaValue.layout;
    });

    const isTreeLayout = computed(() => effectiveLayout.value === 'tree');

    /**
     * Auto layouts position nodes for you, so manual dragging and the selection
     * marquee are disabled while one is active. Derives from the tree flag so
     * there is no second source of truth to keep in sync.
     */
    const isLayoutLocked = isTreeLayout;

    const isConstantWidthEnabled = computed(() => meta.value?.constant_width_enabled ?? false);

    /**
     * Base-unit tree positions, recomputed only when the structure changes
     * (systems, connections, pins, home, the effective mode) — live drag
     * positions and node sizes don't feed in, so it stays out of hot paths.
     */
    const treePositions: ComputedRef<Map<number, Vec2> | null> = computed(() => {
        if (!isTreeLayout.value || !meta.value) return null;
        return computeTreeLayout(toTreeInput(entities, meta.value.home_solarsystem_id), {
            gridSize: view.config.value.grid_size,
        });
    });

    /** The anchor a node renders at: the auto layout when active, else the live position. */
    function renderPosition(id: number): Vec2 | null {
        return treePositions.value?.get(id) ?? entities.positions.get(id) ?? null;
    }

    function resolveConnection(id: number): TResolvedConnection | null {
        const connection = entities.connections.get(id);
        if (!connection) return null;
        const source = entities.systems.get(connection.from_map_solarsystem_id);
        const target = entities.systems.get(connection.to_map_solarsystem_id);
        if (!source || !target) return null;
        return { connection, source, target };
    }

    /** Connections joining two adjacent systems on the active route. */
    const routeConnectionIds: ComputedRef<ReadonlySet<number>> = computed(() => {
        const path = routeDeps.path?.value;
        const ids = new Set<number>();
        if (!path || path.length < 2) return ids;
        const indexBySolarsystem = new Map(path.map((solarsystem, index) => [solarsystem.id, index]));
        for (const connection of entities.connections.values()) {
            const source = entities.systems.get(connection.from_map_solarsystem_id);
            const target = entities.systems.get(connection.to_map_solarsystem_id);
            if (!source || !target) continue;
            const fromIndex = indexBySolarsystem.get(source.solarsystem_id);
            const toIndex = indexBySolarsystem.get(target.solarsystem_id);
            if (fromIndex !== undefined && toIndex !== undefined && Math.abs(fromIndex - toIndex) === 1) {
                ids.add(connection.id);
            }
        }
        return ids;
    });

    /** Connections on the home→rally route, with the direction the animation runs. */
    const rallyEdgeDirections: ComputedRef<ReadonlyMap<number, 'forward' | 'reverse'>> = computed(() => {
        const directions = new Map<number, 'forward' | 'reverse'>();
        const getRallyRouteInfo = routeDeps.getRallyRouteInfo;
        if (!getRallyRouteInfo) return directions;
        for (const connection of entities.connections.values()) {
            const source = entities.systems.get(connection.from_map_solarsystem_id);
            const target = entities.systems.get(connection.to_map_solarsystem_id);
            if (!source || !target) continue;
            const info = getRallyRouteInfo(source.solarsystem_id, target.solarsystem_id);
            if (info.onRoute) {
                directions.set(connection.id, info.reversed ? 'reverse' : 'forward');
            }
        }
        return directions;
    });

    return {
        effectiveLayout,
        isTreeLayout,
        isLayoutLocked,
        isConstantWidthEnabled,
        treePositions,
        renderPosition,
        resolveConnection,
        routeConnectionIds,
        rallyEdgeDirections,
    };
}

/** Translates the entity maps into the structural input the tree layout needs. */
function toTreeInput(entities: EntityState, homeSolarsystemId: number | null): TreeLayoutInput {
    const systems = [...entities.systems.values()];
    const systemsById = entities.systems;

    let fallbackRootId: number | null = null;
    if (homeSolarsystemId !== null) {
        fallbackRootId = systems.find((system) => system.solarsystem_id === homeSolarsystemId)?.id ?? null;
    }

    return {
        nodeIds: systems.map((system) => system.id),
        edges: [...entities.connections.values()].map((connection) => ({
            from: connection.from_map_solarsystem_id,
            to: connection.to_map_solarsystem_id,
        })),
        rootIds: systems.filter((system) => system.pinned).map((system) => system.id),
        fallbackRootId,
        compareNodes: (a: number, b: number): number => {
            const systemA = systemsById.get(a);
            const systemB = systemsById.get(b);
            if (!systemA || !systemB) return 0;
            // The home system always sorts to the top of its level (normally the roots).
            const aHome = systemA.solarsystem_id === homeSolarsystemId;
            const bHome = systemB.solarsystem_id === homeSolarsystemId;
            if (aHome !== bHome) return aHome ? -1 : 1;
            return compareSystems(systemA, systemB);
        },
    };
}
