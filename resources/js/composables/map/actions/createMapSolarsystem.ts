import MapSolarsystems from '@/routes/map-solarsystems';
import { router } from '@inertiajs/vue3';
import { Position } from '@vueuse/core';
import { mapState } from '../state';
import { getFreePosition } from '../utils';

export function createMapSolarsystem(solarsystem_id: number, position: Position | null = null) {
    position = position ?? getFreePosition();
    return router.post(
        MapSolarsystems.store().url,
        {
            map_id: mapState.map!.id,
            solarsystem_id,
            position_x: position.x,
            position_y: position.y,
        },
        {
            preserveState: true,
            preserveScroll: true,
        },
    );
}
