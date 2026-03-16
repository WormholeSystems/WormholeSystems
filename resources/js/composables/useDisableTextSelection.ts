import { useEventListener } from '@vueuse/core';
import { computed, MaybeRefOrGetter, toValue, watch } from 'vue';

/**
 * Composable to disable text selection
 * Useful for grid layouts and other draggable interfaces
 */
export function useDisableTextSelection(shouldPreventValue: MaybeRefOrGetter<boolean>) {
    const shouldPrevent = computed(() => toValue(shouldPreventValue));

    const disableSelection = () => {
        document.documentElement.style.userSelect = 'none';
        document.documentElement.style.webkitUserSelect = 'none';
        window.getSelection()?.removeAllRanges();
    };

    const enableSelection = () => {
        document.documentElement.style.userSelect = '';
        document.documentElement.style.webkitUserSelect = '';
    };

    const preventSelection = (e: Event) => {
        if (shouldPrevent.value) {
            e.preventDefault();
        }
    };

    watch(shouldPrevent, (prevent) => {
        if (prevent) {
            disableSelection();
        } else {
            enableSelection();
        }
    });

    useEventListener(document, 'selectstart', preventSelection, { capture: true, passive: false });

    return {
        disableSelection,
        enableSelection,
    };
}
