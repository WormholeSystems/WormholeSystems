import { useMapStore } from '@/map/store/mapStore';
import MapSelection from '@/routes/map-selection';
import { router } from '@inertiajs/vue3';
import { getSelectedMapSolarsystems } from './selectedMapSolarsystems';

export function deleteSelectedMapSolarsystems(): void {
    const store = useMapStore();
    const selected = getSelectedMapSolarsystems(store);

    if (selected.length === 0) return;

    return router.delete(MapSelection.destroy().url, {
        data: {
            map_solarsystem_ids: selected.map((system) => system.id),
        },
        preserveState: true,
        preserveScroll: true,
        only: ['map', 'map_navigation', 'selected_map_solarsystem'],
        onError: () => router.reload({ only: ['map'] }),
    });
}
