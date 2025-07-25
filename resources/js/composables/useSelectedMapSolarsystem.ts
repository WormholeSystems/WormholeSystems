import { useShowMap } from '@/composables/useShowMap';
import { computed } from 'vue';

export function useSelectedMapSolarsystem() {
    const page = useShowMap();

    return computed(() => page.props.selected_map_solarsystem);
}
