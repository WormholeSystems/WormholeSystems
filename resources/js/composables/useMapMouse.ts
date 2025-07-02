import { useMap } from '@/composables/useMap';
import { useMouse } from '@vueuse/core';
import { computed } from 'vue';

export function useMapMouse() {
    const mouse = useMouse();
    const map = useMap();

    return computed(() => {
        const container = map.value.container!;

        const rect = container.getBoundingClientRect();
        return {
            x: mouse.x.value - rect.left,
            y: mouse.y.value - rect.top,
        };
    });
}
