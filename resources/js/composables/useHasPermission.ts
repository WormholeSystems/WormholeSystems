import { TShowMapProps } from '@/pages/maps';
import { AppPageProps } from '@/types';
import { usePage } from '@inertiajs/vue3';
import { computed } from 'vue';

export function useHasWritePermission() {
    const page = usePage<AppPageProps<TShowMapProps>>();

    return computed(() => page.props.has_write_access);
}
