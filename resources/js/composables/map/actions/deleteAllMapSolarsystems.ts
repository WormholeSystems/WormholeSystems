import MapSelection from '@/routes/map-selection';
import { router } from '@inertiajs/vue3';
import { map_solarsystems } from '../state';

export function deleteAllMapSolarsystems() {
    router.delete(MapSelection.destroy().url, {
        data: {
            map_solarsystem_ids: map_solarsystems.value.filter((s) => !s.pinned).map((s) => s.id),
        },
        preserveState: true,
        preserveScroll: true,
    });
}
