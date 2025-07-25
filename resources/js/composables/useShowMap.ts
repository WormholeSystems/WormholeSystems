import { TShowMapProps } from '@/pages/maps';
import { AppPageProps } from '@/types';
import { usePage } from '@inertiajs/vue3';

export function useShowMap() {
    return usePage<AppPageProps<TShowMapProps>>();
}
