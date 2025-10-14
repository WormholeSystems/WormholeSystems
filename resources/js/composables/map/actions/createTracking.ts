import Tracking from '@/routes/tracking';
import { router } from '@inertiajs/vue3';

export function createTracking(from_map_solarsystem_id: number, to_solarsystem_id: number, signature_id?: number | null) {
    return router.post(
        Tracking.store().url,
        {
            from_map_solarsystem_id,
            to_solarsystem_id,
            signature_id,
        },
        {
            preserveScroll: true,
            preserveState: true,
            only: ['map', 'map_route_solarsystems', 'selected_map_solarsystem'],
        },
    );
}
