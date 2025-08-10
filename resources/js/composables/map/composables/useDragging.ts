import MapSelection from '@/routes/map-selection';
import MapSolarsystems from '@/routes/map-solarsystems';
import { TMapSolarSystem } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { MaybeRefOrGetter, useDraggable, useEventListener } from '@vueuse/core';
import { computed, shallowRef, toValue, watchEffect } from 'vue';
import { mapState, map_solarsystems, map_solarsystems_selected } from '../state';
import { useSelection } from './useSelection';
import { useMapSolarsystems } from './useSolarsystems';

export function useMapSolarsystem(
    system: MaybeRefOrGetter<TMapSolarSystem>,
    element: MaybeRefOrGetter<HTMLElement | null>,
    handle?: MaybeRefOrGetter<HTMLElement | null>,
) {
    const { setSystemPosition } = useMapSolarsystems();
    const { clearSelection } = useSelection();

    const is_dragging = shallowRef(false);

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
        onStart() {
            is_dragging.value = true;
        },
    });

    watchEffect(() => {
        if (draggable.isDragging.value) return;
        if (!current_map_solarsystem.value) return;
        draggable.x.value = current_map_solarsystem.value.position?.x ?? 0;
        draggable.y.value = current_map_solarsystem.value.position?.y ?? 0;
    });

    function handleDragEnd() {
        is_dragging.value = false;
        updateMapSolarsystem(true);
    }

    function handleDrag() {
        const grid_size_map = toValue(mapState.config.grid_size);
        const grid_size = grid_size_map * mapState.scale;
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
                    position_x: draggable.x.value / mapState.scale,
                    position_y: draggable.y.value / mapState.scale,
                    suppress_notification,
                },
                {
                    only: ['map'],
                    preserveState: true,
                    preserveScroll: true,
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
                        position_x: s.position?.x ? s.position.x / mapState.scale : s.position?.x,
                        position_y: s.position?.y ? s.position.y / mapState.scale : s.position?.y,
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

    useEventListener('pointerdown', (event) => {
        if (!is_dragging.value) {
            return;
        }

        event.preventDefault();
    });

    return draggable;
}
