import { useMapStore } from '@/map/store/mapStore';
import MapSelection from '@/routes/map-selection';
import { router } from '@inertiajs/vue3';
import { getOrphanedMapSolarsystems } from './orphanedMapSolarsystems';

export function cleanMapSolarsystems(): void {
    const store = useMapStore();
    const orphaned = getOrphanedMapSolarsystems(store);

    if (orphaned.length === 0) return;

    router.delete(MapSelection.destroy().url, {
        data: {
            map_solarsystem_ids: orphaned.map((system) => system.id),
        },
        preserveState: true,
        preserveScroll: true,
        only: ['map', 'map_navigation', 'selected_map_solarsystem'],
        onError: () => router.reload({ only: ['map'] }),
    });
}
