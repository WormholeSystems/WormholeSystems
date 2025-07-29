import { usePath } from '@/composables/usePath';
import MapSelection from '@/routes/map-selection';
import MapSolarsystems from '@/routes/map-solarsystems';
import { TMapConfig } from '@/types/map';
import { TMap, TMapConnection, TMapSolarSystem } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { MaybeRefOrGetter, useDraggable, useMouse } from '@vueuse/core';
import { computed, reactive, toValue, watchEffect } from 'vue';

type Coordinates = {
    x: number;
    y: number;
};

type TMapState = {
    map: TMap | null;
    map_container: HTMLElement | null;
    map_solarsystems: TDataMapSolarSystem[];
    map_connections: TProcessedConnection[];
    selection: {
        start: Coordinates;
        end: Coordinates | null;
    } | null;
    config: TMapConfig;
    hovered_solarsystem_id: number | null;
};

type WithIsSelected<T> = T & {
    is_selected: boolean;
};

type WithHovered<T> = T & {
    is_hovered: boolean;
};

type TDataMapSolarSystem = WithIsSelected<WithHovered<TMapSolarSystem>>;

const mapState = reactive<TMapState>({
    map: null,
    map_container: null,
    map_solarsystems: [],
    map_connections: [],
    selection: null,
    config: {
        max_size: {
            x: 4000,
            y: 2000,
        },
        grid_size: 20,
    },
    hovered_solarsystem_id: null,
});

const item_height = 40;
const map_solarsystems = computed(() => mapState.map_solarsystems);
const map_solarsystems_selected = computed(() => map_solarsystems.value.filter((s) => s.is_selected && !s.pinned));
const selection = computed(() => mapState.selection);
const grid_size = computed(() => mapState.config.grid_size);

export type TProcessedConnection = TMapConnection & {
    source: TMapSolarSystem;
    target: TMapSolarSystem;
    is_on_route?: boolean;
};

export function useMap(map: MaybeRefOrGetter<TMap>, container: MaybeRefOrGetter<HTMLElement>, config: MaybeRefOrGetter<TMapConfig>) {
    const { path } = usePath();

    watchEffect(() => {
        const mapValue = toValue(map);
        const containerValue = toValue(container);
        if (!mapValue) return;

        const configValue = toValue(config);

        mapState.map = mapValue;
        mapState.map_container = containerValue || null;
        mapState.map_solarsystems = mapValue.map_solarsystems!.map(getSelectedState).map(getHoveredState);
        mapState.config = configValue;
    });

    watchEffect(() => {
        if (!mapState.map) return;
        const mapValue = toValue(mapState.map);
        mapState.map_connections = mapValue.map_connections!.map(getConnectionWithSourceAndTarget);
    });

    function getSelectedState(system: TMapSolarSystem): WithIsSelected<TMapSolarSystem> {
        if (!mapState.selection) return { ...system, is_selected: false };
        if (!mapState.selection.end) return { ...system, is_selected: false };

        const { start, end } = mapState.selection;

        const is_selected =
            system.position!.x >= Math.min(start.x, end.x) &&
            system.position!.x <= Math.max(start.x, end.x) &&
            system.position!.y >= Math.min(start.y, end.y) &&
            system.position!.y <= Math.max(start.y, end.y);

        return { ...system, is_selected };
    }

    function getHoveredState(system: WithIsSelected<TMapSolarSystem>): TDataMapSolarSystem {
        const is_hovered = mapState.hovered_solarsystem_id === system.id;
        return { ...system, is_hovered };
    }

    function getConnectionWithSourceAndTarget(connection: TMapConnection): TProcessedConnection {
        const source = mapState.map_solarsystems.find((s) => s.id === connection.from_map_solarsystem_id)!;
        const target = mapState.map_solarsystems.find((s) => s.id === connection.to_map_solarsystem_id)!;

        const from_map_solarsystem = mapState.map_solarsystems.find((s) => s.id === connection.from_map_solarsystem_id);
        const to_map_solarsystem = mapState.map_solarsystems.find((s) => s.id === connection.to_map_solarsystem_id);
        const from_index = path.value?.findIndex((s) => s.id === from_map_solarsystem?.solarsystem_id) ?? -1;
        const to_index = path.value?.findIndex((s) => s.id === to_map_solarsystem?.solarsystem_id) ?? -1;
        const is_on_route = from_index !== -1 && to_index !== -1 && Math.abs(from_index - to_index) === 1;

        return {
            ...connection,
            source,
            target,
            is_on_route,
        };
    }
}

export function useSelection() {
    function setSelectionStart(x: number, y: number) {
        mapState.selection = {
            start: { x, y },
            end: null,
        };
    }

    function setSelectionEnd(x: number, y: number) {
        if (mapState.selection) {
            mapState.selection.end = { x, y };
        }
    }

    function clearSelection() {
        mapState.selection = null;
    }

    return {
        selection,
        setSelectionStart,
        setSelectionEnd,
        clearSelection,
    };
}

export function useMapSolarsystems() {
    function setSystemPosition(system: TMapSolarSystem, x: number, y: number) {
        const system_state = mapState.map_solarsystems.find((s) => s.id === system.id);
        if (!system_state) return;
        const dx = x - system_state.position!.x;
        const dy = y - system_state.position!.y;
        if (!map_solarsystems_selected.value.length || !map_solarsystems_selected.value.some((s) => s.id === system.id)) {
            if (system_state?.pinned) return;
            system_state.position = { x, y };
            mapState.selection = null;
            return;
        }

        mapState.map_solarsystems.forEach((s) => {
            if (s.is_selected && !s.pinned) {
                s.position = {
                    x: s.position!.x + dx,
                    y: s.position!.y + dy,
                };
            }
        });
    }

    function setHoveredMapSolarsystem(map_solarsystem_id: number, is_hovered: boolean) {
        if (is_hovered) {
            mapState.hovered_solarsystem_id = map_solarsystem_id;
        } else {
            mapState.hovered_solarsystem_id = null;
        }
    }

    return {
        map_solarsystems,
        map_solarsystems_selected,
        setSystemPosition,
        setHoveredMapSolarsystem,
    };
}

export function useMapConnections() {
    return computed(() => mapState.map_connections);
}

export function useMapSolarsystem(
    system: MaybeRefOrGetter<TMapSolarSystem>,
    element: MaybeRefOrGetter<HTMLElement>,
    handle?: MaybeRefOrGetter<HTMLElement>,
) {
    const { setSystemPosition } = useMapSolarsystems();
    const { clearSelection } = useSelection();

    const current_map_solarsystem = computed(() => map_solarsystems.value.find((s) => s.id === toValue(system)?.id)!);

    const draggable = useDraggable(element, {
        initialValue() {
            const systemValue = toValue(system);
            return {
                x: systemValue.position?.x ?? 0,
                y: systemValue.position?.y ?? 0,
            };
        },
        containerElement: () => toValue(mapState.map_container),
        handle,
        onEnd: handleDragEnd,
        onMove: handleDrag,
        disabled() {
            return current_map_solarsystem.value?.pinned;
        },
    });

    watchEffect(() => {
        if (draggable.isDragging.value) return;
        if (!current_map_solarsystem.value) return;
        draggable.x.value = current_map_solarsystem.value.position?.x ?? 0;
        draggable.y.value = current_map_solarsystem.value.position?.y ?? 0;
    });

    function handleDragEnd() {
        updateMapSolarsystem(true);
    }

    function handleDrag() {
        const grid_size = toValue(mapState.config.grid_size);
        let x = Math.round(draggable.x.value / grid_size) * grid_size;
        let y = Math.round(draggable.y.value / grid_size) * grid_size;
        // Ensure the position is within the map boundaries
        const map_width = mapState.config.max_size.x;
        const map_height = mapState.config.max_size.y;
        x = Math.max(40, Math.min(x, map_width - grid_size));
        y = Math.max(20, Math.min(y, map_height - grid_size));
        draggable.x.value = x;
        draggable.y.value = y;

        setSystemPosition(current_map_solarsystem.value, draggable.x.value, draggable.y.value);
    }

    function updateMapSolarsystem(suppress_notification: boolean = false) {
        if (!map_solarsystems_selected.value?.length && !current_map_solarsystem.value?.pinned) {
            return router.put(
                MapSolarsystems.update(current_map_solarsystem.value.id).url,
                {
                    position_x: draggable.x.value,
                    position_y: draggable.y.value,
                    suppress_notification,
                },
                {
                    only: ['map'],
                },
            );
        }

        router.put(
            MapSelection.update().url,
            {
                map_solarsystems: map_solarsystems_selected.value
                    .filter((s) => !s.pinned)
                    .map((s) => ({
                        id: s.id,
                        position_x: s.position?.x,
                        position_y: s.position?.y,
                    })),
            },
            {
                preserveState: true,
                preserveScroll: true,
                onSuccess: () => {
                    clearSelection();
                },
                only: ['map'],
            },
        );
    }

    return draggable;
}

export function useMapMouse() {
    const mouse = useMouse();

    return computed(() => {
        const container = mapState.map_container!;

        const rect = container.getBoundingClientRect();
        return {
            x: mouse.x.value - rect.left - window.scrollX + container.scrollLeft,
            y: mouse.y.value - rect.top - window.scrollY + container.scrollTop,
        };
    });
}

export function useMapGrid() {
    function setMapGridSize(size: number) {
        mapState.config.grid_size = size;
    }

    return {
        grid_size,
        setMapGridSize,
    };
}

export function useMapAction() {
    function removeAllMapSolarsystems() {
        router.delete(MapSelection.destroy().url, {
            data: {
                map_solarsystem_ids: map_solarsystems.value.filter((s) => !s.pinned).map((s) => s.id),
            },
            preserveState: true,
            preserveScroll: true,
        });
    }

    function removeMapSolarsystem(map_solarsystem: TMapSolarSystem) {
        if (map_solarsystem.pinned) return;

        if (map_solarsystems_selected.value.length) {
            return removeSelectedMapSolarsystems();
        }

        return router.delete(MapSolarsystems.destroy(map_solarsystem.id).url, {
            preserveState: true,
            preserveScroll: true,
        });
    }

    function removeSelectedMapSolarsystems() {
        if (map_solarsystems_selected.value.length === 0) return;

        return router.delete(MapSelection.destroy().url, {
            data: {
                map_solarsystem_ids: map_solarsystems_selected.value.map((s) => s.id),
            },
            preserveState: true,
            preserveScroll: true,
        });
    }

    function updateMapSolarsystem(
        map_solarsystem: TMapSolarSystem,
        data: {
            position_x?: number;
            position_y?: number;
            alias?: string;
            occupier_alias?: string;
            status?: string;
            pinned?: boolean;
        },
    ) {
        return router.put(MapSolarsystems.update(map_solarsystem.id).url, data, {
            preserveState: true,
            preserveScroll: true,
            only: ['map'],
        });
    }

    function addMapSolarsystem(solarsystem_id: number) {
        const position = getFreePosition();
        return router.post(
            MapSolarsystems.store().url,
            {
                map_id: mapState.map!.id,
                solarsystem_id,
                position_x: position.x,
                position_y: position.y,
            },
            {
                preserveState: true,
                preserveScroll: true,
            },
        );
    }

    function calculateSortedPositions(sorted_ids: number[]) {
        const sorted_positions = map_solarsystems_selected.value
            .sort((a, b) => {
                if (a.position && b.position) {
                    return a.position.x - b.position.x || a.position.y - b.position.y;
                }
                return 0;
            })
            .map((s) => ({
                ...s,
                position_x: s.position?.x,
                position_y: s.position?.y,
            }));

        return sorted_ids.map((id, index) => ({
            ...sorted_positions[index],
            id,
            position_x: sorted_positions[index].position_x,
            position_y: sorted_positions[index].position_y,
        }));
    }

    function sortByAlias(a: TMapSolarSystem, b: TMapSolarSystem): number {
        if (a.alias && !b.alias) return 1;
        if (!a.alias && b.alias) return -1;

        return a.alias?.localeCompare(b.alias ?? '') || 0;
    }

    function sortByClass(a: TMapSolarSystem, b: TMapSolarSystem): number {
        if (a.class && !b.class) return 1;
        if (!a.class && b.class) return -1;

        const a_security = getSecurityClass(a.solarsystem?.security ?? 0);
        const b_security = getSecurityClass(b.solarsystem?.security ?? 0);

        return a_security.localeCompare(b_security);
    }

    function sortByRegion(a: TMapSolarSystem, b: TMapSolarSystem): number {
        return a.solarsystem?.region?.name.localeCompare(b.solarsystem?.region?.name ?? '') || 0;
    }

    function sortByName(a: TMapSolarSystem, b: TMapSolarSystem): number {
        return a.solarsystem?.name.localeCompare(b.solarsystem?.name ?? '') || 0;
    }

    function getSecurityClass(security: number): string {
        if (security >= 0.5) return 'high';
        if (security >= 0.1) return 'low';
        return 'null';
    }

    function organizeMapSolarsystems(spacing: number = 1) {
        const first_position = map_solarsystems_selected.value.reduce(
            (acc, system) => {
                if (!system.position) return acc;
                return {
                    x: Math.min(acc.x, system.position.x),
                    y: Math.min(acc.y, system.position.y),
                };
            },
            { x: Infinity, y: Infinity },
        );

        const sorted_ids = map_solarsystems_selected.value
            .toSorted(sortByName)
            .toSorted(sortByRegion)
            .toSorted(sortByClass)
            .toSorted(sortByAlias)
            .map((s) => s.id);

        const sorted_positions = calculateSortedPositions(sorted_ids).map((s, index) => {
            const position_x = first_position.x;
            const position_y = first_position.y + index * (spacing * mapState.config.grid_size + item_height);

            return {
                ...s,
                position_x: Math.max(40, Math.min(position_x, mapState.config.max_size.x - mapState.config.grid_size)),
                position_y: Math.max(20, Math.min(position_y, mapState.config.max_size.y - mapState.config.grid_size)),
            };
        });

        router.put(
            MapSelection.update().url,
            {
                map_solarsystems: sorted_positions,
            },
            {
                preserveState: true,
                preserveScroll: true,
                onSuccess: () => {
                    mapState.selection = null;
                },
                only: ['map'],
            },
        );
    }

    return {
        removeAllMapSolarsystems,
        removeMapSolarsystem,
        removeSelectedMapSolarsystems,
        updateMapSolarsystem,
        addMapSolarsystem,
        organizeMapSolarsystems,
    };
}

function getFreePosition(): Coordinates {
    const map_width = mapState.config.max_size.x;
    const map_height = mapState.config.max_size.y;
    const padding = 100; // Padding to avoid edges
    const grid_size = mapState.config.grid_size;
    const boundary_box = {
        x1: -30,
        y1: -30,
        x2: 80,
        y2: 30,
    }; // Relative boundary box for the position of the solar system

    let x = padding;
    let y = padding;

    while (x < map_width - padding) {
        while (y < map_height - padding) {
            const overlaps = map_solarsystems.value.some((s) => {
                const position = { x, y };
                const system_boundary_box = {
                    x1: position.x + boundary_box.x1,
                    y1: position.y + boundary_box.y1,
                    x2: position.x + boundary_box.x2,
                    y2: position.y + boundary_box.y2,
                };

                return (
                    s.position &&
                    s.position.x >= system_boundary_box.x1 &&
                    s.position.x <= system_boundary_box.x2 &&
                    s.position.y >= system_boundary_box.y1 &&
                    s.position.y <= system_boundary_box.y2
                );
            });

            if (!overlaps) {
                return { x, y };
            }

            y += grid_size;
        }

        y = padding;
        x += grid_size;
    }

    throw new Error('No free position found on the map');
}
