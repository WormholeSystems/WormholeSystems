import { useSelectedMapSolarsystem } from '@/composables/useSelectedMapSolarsystem';
import { TMapSolarsystem } from '@/pages/maps';
import MapSelection from '@/routes/map-selection';
import MapSolarsystems from '@/routes/map-solarsystems';
import { router } from '@inertiajs/vue3';
import { useDraggable } from '@vueuse/core';
import { computed, MaybeRefOrGetter, toValue, watchEffect } from 'vue';
import { beginMapDrag, endMapDrag } from '../dragState';
import { map_solarsystems, map_solarsystems_selected, mapState } from '../state';
import { item_anchor_offset, toBaseUnits } from '../utils/position';
import { is_layout_locked } from './useMapViewMode';
import { useSelection } from './useSelection';
import { useMapSolarsystems } from './useSolarsystems';

export function useMapSolarsystem(
    system: MaybeRefOrGetter<TMapSolarsystem>,
    element: MaybeRefOrGetter<HTMLElement | null>,
    handle?: MaybeRefOrGetter<HTMLElement | null>,
) {
    const { setSystemPosition } = useMapSolarsystems();
    const { clearSelection } = useSelection();

    const current_map_solarsystem = computed(() => map_solarsystems.value.find((s) => s.id === toValue(system)?.id)!);

    const selected_map_solarsystem = useSelectedMapSolarsystem();

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
            return current_map_solarsystem.value?.pinned || is_layout_locked.value;
        },
        onStart() {
            beginMapDrag();
        },
    });

    watchEffect(() => {
        if (draggable.isDragging.value) return;
        if (!current_map_solarsystem.value) return;
        draggable.x.value = current_map_solarsystem.value.position?.x ?? 0;
        draggable.y.value = current_map_solarsystem.value.position?.y ?? 0;
    });

    function handleDragEnd() {
        endMapDrag();
        updateMapSolarsystem(true);
    }

    function handleDrag() {
        // Everything here is in scaled pixels (draggable + stored positions), so the
        // grid step and the map bounds must be scaled too — otherwise the clamp is
        // wrong at any zoom other than 100%.
        const scale = mapState.scale;
        const grid_size = mapState.config.grid_size * scale;
        let x = Math.round(draggable.x.value / grid_size) * grid_size;
        let y = Math.round(draggable.y.value / grid_size) * grid_size;
        x = Math.max(item_anchor_offset.x * scale, Math.min(x, mapState.config.max_size.x * scale - grid_size));
        y = Math.max(item_anchor_offset.y * scale, Math.min(y, mapState.config.max_size.y * scale - grid_size));
        draggable.x.value = x;
        draggable.y.value = y;

        setSystemPosition(current_map_solarsystem.value, draggable.x.value, draggable.y.value);
    }

    function updateMapSolarsystem(suppress_notification: boolean = false) {
        if (!map_solarsystems_selected.value?.length && !current_map_solarsystem.value?.pinned) {
            const only = ['map'];
            if (selected_map_solarsystem.value?.id === current_map_solarsystem.value.id) {
                only.push('selected_map_solarsystem');
            }
            return router.put(
                MapSolarsystems.update(current_map_solarsystem.value.id).url,
                {
                    position_x: toBaseUnits(draggable.x.value),
                    position_y: toBaseUnits(draggable.y.value),
                    suppress_notification,
                },
                {
                    only,
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
                        // Guard on the position object, not the coordinate, so an exact 0 is
                        // still scaled back to base units rather than sent through raw.
                        position_x: s.position ? toBaseUnits(s.position.x) : undefined,
                        position_y: s.position ? toBaseUnits(s.position.y) : undefined,
                    })),
            },
            {
                preserveState: true,
                preserveScroll: true,
                onSuccess: () => {
                    clearSelection();
                },
                only: ['map', 'selected_map_solarsystem'],
            },
        );
    }

    return draggable;
}
