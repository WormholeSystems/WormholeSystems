import { TConnectionJump } from '@/pages/maps';
import MapConnectionJumps from '@/routes/map-connection-jumps';
import { router } from '@inertiajs/vue3';

export function deleteMapConnectionJump(jump: TConnectionJump): void {
    return router.delete(MapConnectionJumps.destroy(jump.id).url, {
        preserveScroll: true,
        preserveState: true,
        only: ['map', 'map_navigation'],
        onError: () => router.reload({ only: ['map'] }),
    });
}
