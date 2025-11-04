import { TMapSolarsystem } from '@/pages/maps';
import MapSolarsystems from '@/routes/map-solarsystems';
import { router } from '@inertiajs/vue3';
import { map_solarsystems_selected } from '../state';
import { deleteSelectedMapSolarsystems } from './deleteSelectedMapSolarsystems';

export function deleteMapSolarsystem(map_solarsystem: TMapSolarsystem) {
    if (map_solarsystem.pinned) return;

    if (map_solarsystems_selected.value.length) {
        return deleteSelectedMapSolarsystems();
    }

    return router.delete(MapSolarsystems.destroy(map_solarsystem.id).url, {
        preserveState: true,
        preserveScroll: true,
    });
}
