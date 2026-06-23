import { useSelectedMapSolarsystem } from '@/composables/useSelectedMapSolarsystem';
import { TMapSolarsystem } from '@/pages/maps';
import MapSolarsystems from '@/routes/map-solarsystems';
import { router } from '@inertiajs/vue3';
import { toBaseUnits } from '../utils/position';

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
            // Positions are stored in base units while the in-memory value is scaled
            // (and, in the tree layout, auto-computed). Only send a position when one
            // is actually supplied, so pinning or renaming doesn't overwrite the
            // stored coordinates with the on-screen ones.
            ...(data.position_x !== undefined ? { position_x: toBaseUnits(data.position_x) } : {}),
            ...(data.position_y !== undefined ? { position_y: toBaseUnits(data.position_y) } : {}),
        },
        {
            preserveState: true,
            preserveScroll: true,
            only,
        },
    );
}
