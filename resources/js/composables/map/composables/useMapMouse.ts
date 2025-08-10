import { useMouse } from '@vueuse/core';
import { computed } from 'vue';
import { mapState } from '../state';

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
