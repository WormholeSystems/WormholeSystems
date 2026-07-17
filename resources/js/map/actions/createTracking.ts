import Tracking from '@/routes/tracking';
import { TLifetimeStatus, TMassStatus, TShipSize } from '@/types/models';
import { router } from '@inertiajs/vue3';

type TrackingOptions = {
    signature_id?: number | null;
    alias?: string | null;
    lifetime?: TLifetimeStatus | null;
    mass_status?: TMassStatus | null;
    ship_size?: TShipSize | null;
};

export function createTracking(from_map_solarsystem_id: number, to_solarsystem_id: number, options: TrackingOptions = {}): void {
    return router.post(
        Tracking.store().url,
        {
            from_map_solarsystem_id,
            to_solarsystem_id,
            ...options,
        },
        {
            preserveScroll: true,
            preserveState: true,
            only: ['map', 'map_navigation', 'selected_map_solarsystem'],
            onError: () => router.reload({ only: ['map'] }),
        },
    );
}
