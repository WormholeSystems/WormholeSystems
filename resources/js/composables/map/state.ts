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
export const map_solarsystems_selected = computed(() => map_solarsystems.value.filter((s) => s.is_selected && !s.pinned));
export const selection = computed(() => mapState.selection);
export const grid_size = computed(() => mapState.config.grid_size);
export const scale = computed(() => mapState.scale);
