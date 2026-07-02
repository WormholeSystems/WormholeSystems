import { useMapStore } from '@/map/store/mapStore';
import { TMapSolarsystem } from '@/pages/maps';
import MapSolarsystems from '@/routes/map-solarsystems';
import { router } from '@inertiajs/vue3';
import { deleteSelectedMapSolarsystems } from './deleteSelectedMapSolarsystems';
import { getSelectedMapSolarsystems } from './selectedMapSolarsystems';

export function deleteMapSolarsystem(map_solarsystem: TMapSolarsystem): void {
    if (map_solarsystem.pinned) return;

    const store = useMapStore();

    if (getSelectedMapSolarsystems(store).length) {
        return deleteSelectedMapSolarsystems();
    }

    return router.delete(MapSolarsystems.destroy(map_solarsystem.id).url, {
        preserveState: true,
        preserveScroll: true,
        only: ['map', 'map_navigation', 'selected_map_solarsystem'],
        onError: () => router.reload({ only: ['map'] }),
    });
}
