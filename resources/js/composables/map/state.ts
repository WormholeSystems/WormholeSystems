import { TMapSolarsystem } from '@/pages/maps';
import { computed, reactive } from 'vue';
import { TMapState } from './types';

export const mapState = reactive<TMapState>({
    map: null,
    map_container: null,
    map_solarsystems: [],
    map_connections: [],
    selection: null,
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
 * Whether a system falls inside the current marquee box. Derived from interaction
 * state rather than stored on the system, so the selection can change without
 * rebuilding the system array. Positions and the marquee are both in scaled pixels.
 */
export function isSystemSelected(system: Pick<TMapSolarsystem, 'position'>): boolean {
    const box = mapState.selection;
    if (!box?.end || !system.position) return false;
    const minX = Math.min(box.start.x, box.end.x);
    const maxX = Math.max(box.start.x, box.end.x);
    const minY = Math.min(box.start.y, box.end.y);
    const maxY = Math.max(box.start.y, box.end.y);
    return system.position.x >= minX && system.position.x <= maxX && system.position.y >= minY && system.position.y <= maxY;
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
