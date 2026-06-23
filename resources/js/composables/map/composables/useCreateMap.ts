import { applyScale, getConnectionWithSourceAndTarget, sortByAlias, sortByClass } from '@/composables/map';
import { TLayout } from '@/composables/useLayout';
import { TMap } from '@/pages/maps';
import { TMapConfig } from '@/types/map';
import { computed, MaybeRefOrGetter, toValue, watchEffect } from 'vue';
import { computeTreeLayout } from '../layout/treeLayout';
import { mapState } from '../state';
import { Coordinates, TDataMapSolarSystem } from '../types';
import { TMapLayoutMode } from './useMapViewMode';

export function useCreateMap(
    map: MaybeRefOrGetter<TMap>,
    container: MaybeRefOrGetter<HTMLElement>,
    config: MaybeRefOrGetter<TMapConfig>,
    layout?: MaybeRefOrGetter<TLayout>,
    layoutMode?: MaybeRefOrGetter<TMapLayoutMode>,
) {
    // Base-unit tree positions, recomputed only when the structure changes (systems,
    // connections, pins, home) — not on every hover / selection / zoom, which keep
    // the spanning-forest search out of those hot paths.
    const treeLayout = computed<Map<number, Coordinates> | null>(() => {
        if (toValue(layoutMode) !== 'tree') return null;
        const mapValue = toValue(map);
        if (!mapValue) return null;
        return computeTreeLayout(toTreeInput(mapValue), { gridSize: toValue(config).grid_size });
    });

    watchEffect(createLayout);
    watchEffect(createMap);
    watchEffect(createConnections);

    function createMap() {
        const mapValue = toValue(map);
        const containerValue = toValue(container);
        if (!mapValue) return;

        let solarsystems = mapValue.map_solarsystems!;

        // Tree positions are in base units, like the stored manual positions, so
        // applyScale converts both to pixels uniformly. Selection/hover are no longer
        // baked in here — they derive from interaction state — so this rebuild only
        // runs when the source map, the tree layout, or the scale actually changes.
        const positions = treeLayout.value;
        if (positions) {
            solarsystems = solarsystems.map((system) => {
                const position = positions.get(system.id);
                return position ? { ...system, position } : system;
            });
        }

        mapState.map = mapValue;
        mapState.map_container = containerValue || null;
        mapState.map_solarsystems = solarsystems.map(applyScale);
        mapState.config = toValue(config);
    }

    function createLayout() {
        const layoutValue = toValue(layout);
        mapState.scale = layoutValue?.scale ?? 1;
    }

    function createConnections() {
        const systemsById = new Map(mapState.map_solarsystems.map((system): [number, TDataMapSolarSystem] => [system.id, system]));
        mapState.map_connections = mapState.map!.map_connections!.map((connection) => getConnectionWithSourceAndTarget(connection, systemsById));
    }
}

/** Translates a map into the structural input the tree layout needs. */
function toTreeInput(map: TMap) {
    const systems = map.map_solarsystems ?? [];
    const systemsById = new Map(systems.map((system) => [system.id, system]));

    return {
        nodeIds: systems.map((system) => system.id),
        edges: (map.map_connections ?? []).map((connection) => ({
            from: connection.from_map_solarsystem_id,
            to: connection.to_map_solarsystem_id,
        })),
        rootIds: systems.filter((system) => system.pinned).map((system) => system.id),
        fallbackRootId: systems.find((system) => system.solarsystem_id === map.home_solarsystem_id)?.id ?? null,
        compareNodes: (a: number, b: number): number => {
            const systemA = systemsById.get(a);
            const systemB = systemsById.get(b);
            if (!systemA || !systemB) return 0;
            return sortByAlias(systemA, systemB) || sortByClass(systemA, systemB);
        },
    };
}
