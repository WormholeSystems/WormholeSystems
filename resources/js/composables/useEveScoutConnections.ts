import type { TEveScoutConnection } from '@/types/eve-scout';
import type { AppPageProps } from '@/types/index';
import { usePage } from '@inertiajs/vue3';
import { computed } from 'vue';

export function useEveScoutConnections() {
    const page = usePage<AppPageProps<{ eve_scout_connections?: TEveScoutConnection[] }>>();

    return computed(() => page.props.eve_scout_connections ?? []);
}
