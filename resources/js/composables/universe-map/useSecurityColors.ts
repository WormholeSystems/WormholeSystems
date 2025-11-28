import { useCssVar } from '@vueuse/core';
import { computed, MaybeRefOrGetter, toValue } from 'vue';

/**
 * Provides reactive access to security-related CSS color variables
 * Colors automatically update when theme changes
 */
export function useSecurityColors() {
    const el = typeof document !== 'undefined' ? document.documentElement : null;

    // Security status colors
    const hsColor = useCssVar('--color-hs', el);
    const lsColor = useCssVar('--color-ls', el);
    const nsColor = useCssVar('--color-ns', el);

    // Wormhole class colors
    const c1Color = useCssVar('--color-c1', el);
    const c2Color = useCssVar('--color-c2', el);
    const c3Color = useCssVar('--color-c3', el);
    const c4Color = useCssVar('--color-c4', el);
    const c5Color = useCssVar('--color-c5', el);
    const c6Color = useCssVar('--color-c6', el);

    // Status colors
    const friendlyColor = useCssVar('--color-friendly', el);
    const hostileColor = useCssVar('--color-hostile', el);
    const unknownColor = useCssVar('--color-unknown', el);
    const activeColor = useCssVar('--color-active', el);
    const emptyColor = useCssVar('--color-empty', el);
    const unscanedColor = useCssVar('--color-unscanned', el);

    // Wormhole class color map
    const whClassColors = computed(() => ({
        1: c1Color.value,
        2: c2Color.value,
        3: c3Color.value,
        4: c4Color.value,
        5: c5Color.value,
        6: c6Color.value,
        13: hsColor.value, // Shattered
    }));

    /**
     * Get reactive color for a system based on security/class
     */
    function getColor(security: MaybeRefOrGetter<number>, wormholeClass: MaybeRefOrGetter<number | null>) {
        return computed(() => {
            const whClass = toValue(wormholeClass);
            const sec = toValue(security);

            if (whClass !== null) {
                return whClassColors.value[whClass as keyof typeof whClassColors.value] ?? unknownColor.value;
            }

            if (sec >= 0.5) return hsColor.value;
            if (sec >= 0.1) return lsColor.value;
            return nsColor.value;
        });
    }

    return {
        // Individual colors
        hsColor,
        lsColor,
        nsColor,
        c1Color,
        c2Color,
        c3Color,
        c4Color,
        c5Color,
        c6Color,
        friendlyColor,
        hostileColor,
        unknownColor,
        activeColor,
        emptyColor,
        unscanedColor,
        // Computed maps
        whClassColors,
        // Helper function
        getColor,
    };
}
