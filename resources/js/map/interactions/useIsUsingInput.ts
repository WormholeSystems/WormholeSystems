import { useActiveElement } from '@vueuse/core';
import { computed } from 'vue';

/** Whether the keyboard focus sits in a text input, so global map hotkeys (Delete, …) stay quiet. */
export function useIsUsingInput() {
    const activeElement = useActiveElement();
    return computed(() => {
        return Boolean(activeElement.value && ['INPUT', 'TEXTAREA'].includes(activeElement.value.tagName));
    });
}
