import MapSolarsystems from '@/routes/map-solarsystems';
import { router } from '@inertiajs/vue3';
import { Position } from '@vueuse/core';
import { mapState } from '../state';
import { getFreePosition } from '../utils';

/**
 * Adds a system to the map. When `connectToMapSolarsystemId` is given, the backend also
 * links the new system to it in the same request (the "Add connection" flow), so the
 * frontend never needs the new node's id.
 */
export function createMapSolarsystem(solarsystem_id: number, position: Position | null = null, connectToMapSolarsystemId: number | null = null) {
    position = position ?? getFreePosition();
    return router.post(
        MapSolarsystems.store().url,
        {
            map_id: mapState.map!.id,
            solarsystem_id,
            position_x: position.x,
            position_y: position.y,
            ...(connectToMapSolarsystemId != null ? { connect_to_map_solarsystem_id: connectToMapSolarsystemId } : {}),
        },
        {
            preserveState: true,
            preserveScroll: true,
        },
    );
}
