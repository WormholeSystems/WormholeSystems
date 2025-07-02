import { useMapMouse } from '@/composables/useMapMouse';
import { useEventListener } from '@vueuse/core';
import { computed, reactive } from 'vue';

type SelectionStore = {
    start: { x: number; y: number } | null;
    area?: { x1: number; y1: number; x2: number; y2: number } | null;
};

const selection_store = reactive<SelectionStore>({
    start: null,
});

export function useSelection() {
    const start = computed(() => selection_store.start);
    const area = computed(() => selection_store.area);

    const mouse = useMapMouse();

    function handleDragStart() {
        selection_store.start = { x: mouse.value.x, y: mouse.value.y };
    }

    function handleDragEnd() {
        if (selection_store.start) {
            selection_store.area = {
                x1: selection_store.start.x,
                y1: selection_store.start.y,
                x2: mouse.value.x,
                y2: mouse.value.y,
            };

            console.log('Selection area:', selection_store.area);

            selection_store.start = null;
        }
    }

    function handleDragMove() {
        if (selection_store.start) {
            selection_store.area = {
                x1: selection_store.start.x,
                y1: selection_store.start.y,
                x2: mouse.value.x,
                y2: mouse.value.y,
            };
        }
    }

    useEventListener('mouseup', handleDragEnd);

    return {
        start,
        area,
        handleDragStart,
        handleDragEnd,
        handleDragMove,
    };
}

export function useSelectionArea() {
    return computed(() => selection_store.area);
}
