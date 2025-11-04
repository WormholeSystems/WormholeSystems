import { useSelectedMapSolarsystem } from '@/composables/useSelectedMapSolarsystem';
import { TMapSolarsystem } from '@/pages/maps';
import MapSolarsystems from '@/routes/map-solarsystems';
import { router } from '@inertiajs/vue3';
import { mapState } from '../state';

export function updateMapSolarsystem(
    map_solarsystem: TMapSolarsystem,
    data: {
        position_x?: number;
        position_y?: number;
        alias?: string;
        occupier_alias?: string;
        status?: string;
        pinned?: boolean;
    },
) {
    const only = ['map'];

    const selected = useSelectedMapSolarsystem();

    if (selected.value?.id === map_solarsystem.id) {
        only.push('selected_map_solarsystem');
    }
    return router.put(
        MapSolarsystems.update(map_solarsystem.id).url,
        {
            ...data,
            position_x: data.position_x ? data.position_x * (1 / mapState.scale) : map_solarsystem.position?.x,
            position_y: data.position_y ? data.position_y * (1 / mapState.scale) : map_solarsystem.position?.y,
        },
        {
            preserveState: true,
            preserveScroll: true,
            only,
        },
    );
}
