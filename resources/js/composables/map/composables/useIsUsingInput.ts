import { useActiveElement } from '@vueuse/core';
import { computed } from 'vue';

export function useIsUsingInput() {
    const activeElement = useActiveElement();
    return computed(() => {
        return activeElement.value && ['INPUT', 'TEXTAREA'].includes(activeElement.value.tagName);
    });
}
