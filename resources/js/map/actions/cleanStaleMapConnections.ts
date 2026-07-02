import BulkMapConnectionController from '@/actions/App/Http/Controllers/BulkMapConnectionController';
import type { TMap } from '@/pages/maps';
import { router } from '@inertiajs/vue3';

/**
 * Actions never reload the 'map' prop: the sync layer patches the store from
 * broadcast events instead. On a failed request the store may hold state the
 * server rejected, so we resync it with a one-off 'map' reload.
 */
export function cleanStaleMapConnections(map: Pick<TMap, 'slug'>): void {
    return router.delete(BulkMapConnectionController.destroy(map.slug).url, {
        preserveScroll: true,
        preserveState: true,
        only: ['map', 'map_navigation'],
        onError: () => router.reload({ only: ['map'] }),
    });
}
