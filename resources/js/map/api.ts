import { useMapStore, type MapStore } from '@/map/store/mapStore';
import type { TMapConnection, TMapSolarsystem } from '@/pages/maps';
import { computed, type ComputedRef } from 'vue';

/**
 * The public surface of the map subsystem. Code outside resources/js/map must
 * import from this module only — everything else in map/ is internal and free
 * to change shape.
 */

export { useMapStore } from '@/map/store/mapStore';
export type { MapStore } from '@/map/store/mapStore';

export { cleanStaleMapConnections } from '@/map/actions/cleanStaleMapConnections';
export { createMapSolarsystem } from '@/map/actions/createMapSolarsystem';
export { createSignature } from '@/map/actions/createSignature';
export { createTracking } from '@/map/actions/createTracking';
export { deleteSignature } from '@/map/actions/deleteSignature';
export { deleteSignatures } from '@/map/actions/deleteSignatures';
export { pasteSignatures } from '@/map/actions/pasteSignatures';
export { updateMapConnection } from '@/map/actions/updateMapConnection';
export { updateMapUserSettings } from '@/map/actions/updateMapUserSettings';
export { updateSignature } from '@/map/actions/updateSignature';

export { useIsUsingInput } from '@/map/interactions/useIsUsingInput';
export { useSolarsystemSearch } from '@/map/interactions/useSolarsystemSearch';

/**
 * A connection joined with its endpoint systems, as the signatures panel and the
 * landing fixtures consume it. Matches the shape the old map tree produced.
 */
export type TProcessedConnection = TMapConnection & {
    source: TMapSolarsystem;
    target: TMapSolarsystem;
    is_on_route?: boolean;
    is_on_rally_route?: boolean;
    rally_route_reversed?: boolean;
};

/** The current map store, or null while no map canvas has been mounted yet. */
function resolveMapStore(): MapStore | null {
    try {
        return useMapStore();
    } catch {
        return null;
    }
}

/**
 * Whether any map gesture (pan, marquee, node drag, link drag) is in progress.
 * Resolves the store lazily inside the computed so it can be created before the
 * map canvas mounts (it reports false until a store exists).
 */
export function useIsMapGestureActive(): ComputedRef<boolean> {
    return computed(() => (resolveMapStore()?.activeGesture.value ?? 'none') !== 'none');
}

/**
 * Sidebar-facing view of the systems on the map: the list of map solarsystems
 * plus the hover setter, resolved lazily against the current map store. Both
 * degrade gracefully (empty list, no-op) while no map canvas is mounted.
 */
export function useMapSolarsystems(): {
    map_solarsystems: ComputedRef<TMapSolarsystem[]>;
    setHoveredMapSolarsystem: (map_solarsystem_id: number, is_hovered: boolean) => void;
} {
    const map_solarsystems = computed<TMapSolarsystem[]>(() => {
        const store = resolveMapStore();
        return store ? [...store.systems.values()] : [];
    });

    function setHoveredMapSolarsystem(map_solarsystem_id: number, is_hovered: boolean): void {
        const store = resolveMapStore();
        if (!store) return;
        store.hoveredSolarsystemId.value = is_hovered ? map_solarsystem_id : null;
    }

    return { map_solarsystems, setHoveredMapSolarsystem };
}
