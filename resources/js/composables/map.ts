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
    map_solarsystems: WithIsSelected<TMapSolarSystem>[];
    map_connections: TConnectionWithSourceAndTarget[];
    selection: {
        start: Coordinates;
        end: Coordinates | null;
    } | null;
};

type WithIsSelected<T> = T & {
    is_selected: boolean;
};

const mapState = reactive<TMapState>({
    map: null,
    map_container: null,
    map_solarsystems: [],
    map_connections: [],
    selection: null,
});

const map_solarsystems = computed(() => mapState.map_solarsystems);
const map_solarsystems_selected = computed(() => map_solarsystems.value.filter((s) => s.is_selected));
const selection = computed(() => mapState.selection);

export type TConnectionWithSourceAndTarget = TMapConnection & {
    source: TMapSolarSystem;
    target: TMapSolarSystem;
};

export function useMap(map: MaybeRefOrGetter<TMap>, container?: MaybeRefOrGetter<HTMLElement>) {
    watchEffect(() => {
        const mapValue = toValue(map);
        const containerValue = toValue(container);
        if (!mapValue) return;

        mapState.map = mapValue;
        mapState.map_container = containerValue || null;
        mapState.map_solarsystems = mapValue.map_solarsystems.map(getSelectedState);
    });

    watchEffect(() => {
        if (!mapState.map) return;
        const mapValue = toValue(mapState.map);
        mapState.map_connections = mapValue.map_connections.map(getConnectionWithSourceAndTarget);
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

    function getConnectionWithSourceAndTarget(connection: TMapConnection): TConnectionWithSourceAndTarget {
        const source = mapState.map_solarsystems.find((s) => s.id === connection.from_map_solarsystem_id)!;
        const target = mapState.map_solarsystems.find((s) => s.id === connection.to_map_solarsystem_id)!;

        return {
            ...connection,
            source,
            target,
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
        const selected_systems = mapState.map_solarsystems.filter((s) => s.is_selected);
        if (!system_state) return;
        const dx = x - system_state.position!.x;
        const dy = y - system_state.position!.y;
        if (!selected_systems.length || !selected_systems.some((s) => s.id === system.id)) {
            system_state.position = { x, y };
            mapState.selection = null;
            return;
        }

        mapState.map_solarsystems.forEach((s) => {
            if (s.is_selected) {
                s.position = {
                    x: s.position!.x + dx,
                    y: s.position!.y + dy,
                };
            }
        });
    }

    return {
        map_solarsystems,
        map_solarsystems_selected,
        setSystemPosition,
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
    });

    watchEffect(() => {
        if (draggable.isDragging.value) return;
        console.log('Updating draggable position for system', current_map_solarsystem.value.id);
        draggable.x.value = current_map_solarsystem.value.position?.x ?? 0;
        draggable.y.value = current_map_solarsystem.value.position?.y ?? 0;
    });

    function handleDragEnd() {
        updateMapSolarsystem();
    }

    function handleDrag() {
        setSystemPosition(current_map_solarsystem.value, draggable.x.value, draggable.y.value);
    }

    function updateMapSolarsystem() {
        if (!map_solarsystems_selected.value.length) {
            return router.put(route('map-solarsystems.update', current_map_solarsystem.value.id), {
                position_x: draggable.x.value,
                position_y: draggable.y.value,
            });
        }

        router.put(
            route('map-selection.update'),
            {
                map_solarsystems: map_solarsystems_selected.value.map((s) => ({
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
            x: mouse.x.value - rect.left,
            y: mouse.y.value - rect.top,
        };
    });
}
