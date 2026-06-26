import BulkMapConnectionController from '@/actions/App/Http/Controllers/BulkMapConnectionController';
import type { TMap } from '@/pages/maps';
import { router } from '@inertiajs/vue3';

export function cleanStaleMapConnections(map: TMap) {
    return router.delete(BulkMapConnectionController.destroy(map.slug).url, {
        preserveScroll: true,
        preserveState: true,
        only: ['map', 'map_navigation'],
    });
}
