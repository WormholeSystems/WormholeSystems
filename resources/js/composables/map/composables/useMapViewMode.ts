import { useStorage } from '@vueuse/core';
import { computed } from 'vue';

export type TMapLayoutMode = 'manual' | 'tree';

// Module-level singletons so every component (the map, the options pill, …) reads
// and writes the same persisted value rather than separate, out-of-sync refs.
const viewMode = useStorage<TMapLayoutMode>('map-view-mode', 'manual');
const is_tree_layout = computed(() => viewMode.value === 'tree');

/**
 * Auto layouts (the tree view) position nodes for you, so manual dragging and the
 * selection marquee are disabled while one is active. Exported for the low-level
 * composables that only care about the lock; it derives from {@link is_tree_layout}
 * so there is no second source of truth to keep in sync.
 */
export const is_layout_locked = is_tree_layout;

export function useMapViewMode() {
    return { viewMode, is_tree_layout };
}
