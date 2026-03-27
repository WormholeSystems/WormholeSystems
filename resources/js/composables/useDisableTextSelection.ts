import { useEventListener } from '@vueuse/core';
import { computed, MaybeRefOrGetter, toValue, watch } from 'vue';
import { isSSR } from './useOnClient';

/**
 * Composable to disable text selection
 * Useful for grid layouts and other draggable interfaces
 */
export function useDisableTextSelection(shouldPreventValue: MaybeRefOrGetter<boolean>) {
    const shouldPrevent = computed(() => toValue(shouldPreventValue));

    const disableSelection = () => {
        if (isSSR()) return;
        document.documentElement.style.userSelect = 'none';
        document.documentElement.style.webkitUserSelect = 'none';
        window.getSelection()?.removeAllRanges();
    };

    const enableSelection = () => {
        if (isSSR()) return;
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

    if (!isSSR()) {
        useEventListener(document, 'selectstart', preventSelection, { capture: true, passive: false });
    }

    return {
        disableSelection,
        enableSelection,
    };
}
