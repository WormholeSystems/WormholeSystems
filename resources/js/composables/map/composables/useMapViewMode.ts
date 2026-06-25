import { computed } from 'vue';
import { mapState } from '../state';

export type TMapLayoutMode = 'manual' | 'tree';

// The map's layout is the shared default a manager sets. When the map allows it, a viewer
// can override it for themselves (mapState.user_layout_override). The effective mode the
// rest of the map reacts to is the override when present, otherwise the map default.
const effective_layout = computed<TMapLayoutMode>(() => {
    const map = mapState.map;
    if (!map) return 'manual';
    if (map.allow_layout_override && mapState.user_layout_override) {
        return mapState.user_layout_override;
    }
    return map.layout;
});

const is_tree_layout = computed(() => effective_layout.value === 'tree');

/**
 * Auto layouts (the tree view) position nodes for you, so manual dragging and the
 * selection marquee are disabled while one is active. Exported for the low-level
 * composables that only care about the lock; it derives from {@link is_tree_layout}
 * so there is no second source of truth to keep in sync.
 */
export const is_layout_locked = is_tree_layout;

export function useMapViewMode() {
    return { is_tree_layout, effective_layout };
}
