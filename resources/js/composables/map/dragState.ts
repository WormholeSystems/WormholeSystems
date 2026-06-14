import { computed, reactive } from 'vue';

/**
 * Tracks how many map drag interactions are currently active (moving a system
 * node, drawing a new connection, etc.). Driving a single shared flag lets us
 * reuse `useDisableTextSelection` to block accidental text selection during any
 * of them, the same way the layout editor does.
 */
const state = reactive({ active_drags: 0 });

export const is_map_dragging = computed(() => state.active_drags > 0);

export function beginMapDrag(): void {
    state.active_drags++;
}

export function endMapDrag(): void {
    state.active_drags = Math.max(0, state.active_drags - 1);
}
