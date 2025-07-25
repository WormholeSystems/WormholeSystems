import { useShowMap } from '@/composables/useShowMap';
import { computed } from 'vue';

export function useMap() {
    const page = useShowMap();

    return computed(() => page.props.map);
}
