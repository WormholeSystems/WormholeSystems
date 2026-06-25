import { computed } from 'vue';
import { mapState } from '../state';

export type TMapLayoutMode = 'manual' | 'tree';

// The layout mode is a per-map setting (managers toggle it; everyone shares it), so it is
// derived from the live map rather than a per-browser value.
const is_tree_layout = computed(() => mapState.map?.layout === 'tree');

/**
 * Auto layouts (the tree view) position nodes for you, so manual dragging and the
 * selection marquee are disabled while one is active. Exported for the low-level
 * composables that only care about the lock; it derives from {@link is_tree_layout}
 * so there is no second source of truth to keep in sync.
 */
export const is_layout_locked = is_tree_layout;

export function useMapViewMode() {
    return { is_tree_layout };
}
