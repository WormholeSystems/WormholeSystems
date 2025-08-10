import MapSelection from '@/routes/map-selection';
import { router } from '@inertiajs/vue3';
import { map_solarsystems_selected } from '../state';

export function deleteSelectedMapSolarsystems() {
    if (map_solarsystems_selected.value.length === 0) return;

    return router.delete(MapSelection.destroy().url, {
        data: {
            map_solarsystem_ids: map_solarsystems_selected.value.map((s) => s.id),
        },
        preserveState: true,
        preserveScroll: true,
    });
}
