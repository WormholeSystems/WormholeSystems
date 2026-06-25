import { TMapSolarsystem } from '@/pages/maps';
import { computed, reactive } from 'vue';
import { TMapState } from './types';

export const mapState = reactive<TMapState>({
    map: null,
    map_container: null,
    map_solarsystems: [],
    map_connections: [],
    selection: null,
    selected_ids: [],
    user_layout_override: null,
    config: {
        max_size: {
            x: 4000,
            y: 2000,
        },
        grid_size: 20,
    },
    scale: 1,
    hovered_solarsystem_id: null,
});

export const map_solarsystems = computed(() => mapState.map_solarsystems);

/**
 * Whether a system is part of the current selection. The selected set is a sticky
 * list of ids (committed from the marquee) rather than a live box test, so dragging
 * the systems out of the box doesn't deselect them mid-drag.
 */
export function isSystemSelected(system: Pick<TMapSolarsystem, 'id'>): boolean {
    return mapState.selected_ids.includes(system.id);
}

/**
 * Commit the systems currently inside the marquee box to the selected set. Called as
 * the box is drawn, so the highlight tracks the marquee live. Positions and the
 * marquee are both in scaled pixels.
 */
export function selectSystemsInBox(): void {
    const box = mapState.selection;
    if (!box?.end) {
        mapState.selected_ids = [];
        return;
    }
    const minX = Math.min(box.start.x, box.end.x);
    const maxX = Math.max(box.start.x, box.end.x);
    const minY = Math.min(box.start.y, box.end.y);
    const maxY = Math.max(box.start.y, box.end.y);
    mapState.selected_ids = map_solarsystems.value
        .filter((s) => s.position && s.position.x >= minX && s.position.x <= maxX && s.position.y >= minY && s.position.y <= maxY)
        .map((s) => s.id);
}

/** Whether a system is the currently hovered one. */
export function isSystemHovered(id: number): boolean {
    return mapState.hovered_solarsystem_id === id;
}

export const map_solarsystems_selected = computed(() =>
    map_solarsystems.value.filter((s) => isSystemSelected(s) && !s.pinned && s.solarsystem_id !== mapState.map?.home_solarsystem_id),
);
export const selection = computed(() => mapState.selection);
export const grid_size = computed(() => mapState.config.grid_size);
export const scale = computed(() => mapState.scale);
