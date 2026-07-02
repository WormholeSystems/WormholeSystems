import { useMapStore } from '@/map/store/mapStore';
import MapSelection from '@/routes/map-selection';
import { router } from '@inertiajs/vue3';

/**
 * Persists the positions of a group-dragged selection in one request. The map
 * itself was already moved optimistically in the store; on success the sticky
 * selection is dropped (matching the old group-drag flow), on failure the map
 * prop is resynced so the nodes snap back to server truth.
 */
export function updateMapSelection(positions: { id: number; position_x: number; position_y: number }[]): void {
    const store = useMapStore();

    if (positions.length === 0) return;

    return router.put(
        MapSelection.update().url,
        { map_solarsystems: positions },
        {
            preserveState: true,
            preserveScroll: true,
            only: ['map'],
            onSuccess: () => store.clearSelection(),
            onError: () => router.reload({ only: ['map'] }),
        },
    );
}
