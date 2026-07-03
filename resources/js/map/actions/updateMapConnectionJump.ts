import { TConnectionJump } from '@/pages/maps';
import MapConnectionJumps from '@/routes/map-connection-jumps';
import { router } from '@inertiajs/vue3';
import { TMapConnectionJumpDirection } from './createMapConnectionJump';

export function updateMapConnectionJump(
    jump: TConnectionJump,
    data: {
        direction?: TMapConnectionJumpDirection;
        ship_type_id?: number | null;
        mass?: number;
    },
    options: { onSuccess?: () => void; onError?: (errors: Record<string, string>) => void } = {},
): void {
    return router.put(MapConnectionJumps.update(jump.id).url, data, {
        preserveScroll: true,
        preserveState: true,
        only: ['map', 'map_navigation'],
        onSuccess: options.onSuccess,
        onError: (errors) => {
            options.onError?.(errors);
            void router.reload({ only: ['map'] });
        },
    });
}
