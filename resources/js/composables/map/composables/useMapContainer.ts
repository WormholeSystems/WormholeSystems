import { mapState } from '@/composables/map';
import { computed } from 'vue';

export function useMapContainer() {
    return computed(() => mapState.map_container);
}
