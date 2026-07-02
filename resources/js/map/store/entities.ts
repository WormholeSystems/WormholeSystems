import type { Size, Vec2 } from '@/map/core/types';
import { TMap, TMapConnection, TMapSolarsystem } from '@/pages/maps';
import { shallowReactive } from 'vue';

/**
 * Entity state: server truth keyed by id in shallow-reactive Maps, so Vue tracks
 * per key and an update to one entity never invalidates readers of another.
 * Entity objects are replaced, never mutated — identity equality means unchanged.
 *
 * Live render positions are a separate Map so the drag hot path invalidates only
 * geometry readers, not the node cards reading the entity objects.
 */
export type SystemCounts = {
    signatures_count: number;
    wormhole_signatures_count: number;
    uncategorized_signatures_count: number;
};

export type EntityState = ReturnType<typeof createEntityState>;

export function createEntityState() {
    const systems = shallowReactive(new Map<number, TMapSolarsystem>());
    const connections = shallowReactive(new Map<number, TMapConnection>());
    /** Live anchor positions in base units — server truth plus in-flight drags. */
    const positions = shallowReactive(new Map<number, Vec2>());
    /** Measured node sizes in base units, written by the measurement observer. */
    const nodeSizes = shallowReactive(new Map<number, Size>());
    /**
     * Systems whose position is owned by an active drag: remote updates for them
     * apply every field except the position, so an incoming echo can't yank the
     * node out from under the pointer.
     */
    const lockedPositionIds = new Set<number>();

    function upsertSystem(system: TMapSolarsystem): void {
        systems.set(system.id, system);
        if (system.position && !lockedPositionIds.has(system.id)) {
            positions.set(system.id, { x: system.position.x, y: system.position.y });
        }
    }

    function removeSystem(id: number): void {
        systems.delete(id);
        positions.delete(id);
        nodeSizes.delete(id);
        lockedPositionIds.delete(id);
    }

    function upsertConnection(connection: TMapConnection): void {
        connections.set(connection.id, connection);
    }

    function removeConnection(id: number): void {
        connections.delete(id);
    }

    /** Local-only position write for the drag hot path; entity objects stay untouched. */
    function moveSystem(id: number, position: Vec2): void {
        positions.set(id, position);
    }

    function patchSystemCounts(id: number, counts: SystemCounts): void {
        const system = systems.get(id);
        if (!system) return;
        systems.set(id, { ...system, ...counts });
    }

    function lockPosition(id: number): void {
        lockedPositionIds.add(id);
    }

    function unlockPosition(id: number): void {
        lockedPositionIds.delete(id);
    }

    /**
     * Keyed diff against a full map payload: upserts entities that changed,
     * removes ids that disappeared, and leaves unchanged entities referentially
     * identical so nothing downstream recomputes for them.
     */
    function reconcile(map: Pick<TMap, 'map_solarsystems' | 'map_connections'>): void {
        const incomingSystems = new Set<number>();
        for (const system of map.map_solarsystems ?? []) {
            incomingSystems.add(system.id);
            if (!entityEquals(systems.get(system.id), system)) {
                upsertSystem(system);
            }
        }
        for (const id of [...systems.keys()]) {
            if (!incomingSystems.has(id)) {
                removeSystem(id);
            }
        }

        const incomingConnections = new Set<number>();
        for (const connection of map.map_connections ?? []) {
            incomingConnections.add(connection.id);
            if (!entityEquals(connections.get(connection.id), connection)) {
                upsertConnection(connection);
            }
        }
        for (const id of [...connections.keys()]) {
            if (!incomingConnections.has(id)) {
                removeConnection(id);
            }
        }
    }

    return {
        systems,
        connections,
        positions,
        nodeSizes,
        lockedPositionIds,
        upsertSystem,
        removeSystem,
        upsertConnection,
        removeConnection,
        moveSystem,
        patchSystemCounts,
        lockPosition,
        unlockPosition,
        reconcile,
    };
}

/**
 * Structural equality between an existing entity and its incoming version.
 * Reference equality fast-paths the common case (the static solarsystem data is
 * cached, so unchanged sub-objects keep their identity across prop reloads);
 * fresh-but-equal sub-objects fall back to a JSON comparison.
 */
function entityEquals(a: object | undefined, b: object): boolean {
    if (!a) return false;
    if (a === b) return true;
    const aKeys = Object.keys(a);
    const bKeys = Object.keys(b);
    if (aKeys.length !== bKeys.length) return false;
    for (const key of aKeys) {
        const aValue = (a as Record<string, unknown>)[key];
        const bValue = (b as Record<string, unknown>)[key];
        if (Object.is(aValue, bValue)) continue;
        if (typeof aValue !== 'object' || typeof bValue !== 'object' || aValue === null || bValue === null) return false;
        if (JSON.stringify(aValue) !== JSON.stringify(bValue)) return false;
    }
    return true;
}
