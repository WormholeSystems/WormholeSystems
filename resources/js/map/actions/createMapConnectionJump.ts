import MapConnectionJumps from '@/routes/map-connection-jumps';
import { router } from '@inertiajs/vue3';

export type TMapConnectionJumpDirection = 'outbound' | 'inbound';

export function createMapConnectionJump(
    map_connection_id: number,
    data: {
        direction: TMapConnectionJumpDirection;
        ship_type_id?: number | null;
        mass?: number | null;
    },
    options: { onSuccess?: () => void; onError?: (errors: Record<string, string>) => void } = {},
): void {
    return router.post(
        MapConnectionJumps.store().url,
        { map_connection_id, ...data },
        {
            preserveScroll: true,
            preserveState: true,
            only: ['map', 'map_navigation'],
            onSuccess: options.onSuccess,
            onError: (errors) => {
                options.onError?.(errors);
                void router.reload({ only: ['map'] });
            },
        },
    );
}
