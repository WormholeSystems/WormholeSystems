import { findFreePosition } from '@/map/core/layout/freePlacement';
import type { Vec2 } from '@/map/core/types';
import { useMapStore } from '@/map/store/mapStore';
import MapSolarsystems from '@/routes/map-solarsystems';
import { router } from '@inertiajs/vue3';

/**
 * Adds a system to the map. When `connectToMapSolarsystemId` is given, the backend also
 * links the new system to it in the same request (the "Add connection" flow), so the
 * frontend never needs the new node's id.
 */
export function createMapSolarsystem(solarsystem_id: number, position: Vec2 | null = null, connectToMapSolarsystemId: number | null = null): void {
    const store = useMapStore();
    const meta = store.meta.value;
    if (!meta) {
        throw new Error('createMapSolarsystem() called before the map store was hydrated');
    }

    position =
        position ??
        findFreePosition([...store.positions.values()], {
            maxSize: store.config.value.max_size,
            gridSize: store.config.value.grid_size,
        });

    return router.post(
        MapSolarsystems.store().url,
        {
            map_id: meta.id,
            solarsystem_id,
            position_x: position.x,
            position_y: position.y,
            ...(connectToMapSolarsystemId != null ? { connect_to_map_solarsystem_id: connectToMapSolarsystemId } : {}),
        },
        {
            preserveState: true,
            preserveScroll: true,
            only: ['map', 'map_navigation', 'selected_map_solarsystem'],
            onError: () => router.reload({ only: ['map'] }),
        },
    );
}
