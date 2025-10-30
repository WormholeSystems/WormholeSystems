import { useEventListener } from '@vueuse/core';
import { computed, MaybeRefOrGetter, toValue, watch } from 'vue';

/**
 * Composable to disable text selection
 * Useful for grid layouts and other draggable interfaces
 */
export function useDisableTextSelection(shouldPreventValue: MaybeRefOrGetter<boolean>) {
    const shouldPrevent = computed(() => toValue(shouldPreventValue));
    const disableSelection = () => {
        document.body.classList.add('select-none');
    };

    const enableSelection = () => {
        document.body.classList.remove('select-none');
    };

    // Event handlers
    const forcePreventSelection = (e: Event) => {
        if (shouldPrevent.value) {
            e.preventDefault();
            e.stopPropagation();
            e.stopImmediatePropagation();
            return false;
        }
        return true;
    };

    // Watch for changes and toggle selection
    watch(shouldPrevent, (prevent) => {
        if (prevent) {
            disableSelection();
        } else {
            enableSelection();
        }
    });

    useEventListener('selectstart', forcePreventSelection, { passive: false });
    useEventListener('dragstart', forcePreventSelection, { passive: false });
    return {
        disableSelection,
        enableSelection,
    };
}
