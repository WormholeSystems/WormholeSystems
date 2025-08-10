import { computed } from 'vue';
import { mapState } from '../state';

export function useMapConnections() {
    return computed(() => mapState.map_connections);
}
