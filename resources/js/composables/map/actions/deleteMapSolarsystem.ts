import MapSolarsystems from '@/routes/map-solarsystems';
import { TMapSolarSystem } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { map_solarsystems_selected } from '../state';
import { deleteSelectedMapSolarsystems } from './deleteSelectedMapSolarsystems';

export function deleteMapSolarsystem(map_solarsystem: TMapSolarSystem) {
    if (map_solarsystem.pinned) return;

    if (map_solarsystems_selected.value.length) {
        return deleteSelectedMapSolarsystems();
    }

    return router.delete(MapSolarsystems.destroy(map_solarsystem.id).url, {
        preserveState: true,
        preserveScroll: true,
    });
}
