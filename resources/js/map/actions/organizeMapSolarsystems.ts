import { organizeIntoColumn } from '@/map/core/layout/organize';
import { compareSystems } from '@/map/core/sorting';
import { useMapStore } from '@/map/store/mapStore';
import MapSelection from '@/routes/map-selection';
import { router } from '@inertiajs/vue3';
import { getSelectedMapSolarsystems } from './selectedMapSolarsystems';

export function organizeMapSolarsystems(spacing: number = 1): void {
    const store = useMapStore();
    const selected = getSelectedMapSolarsystems(store);

    if (selected.length === 0) return;

    const orderedIds = selected.toSorted(compareSystems).map((system) => system.id);
    const positions = organizeIntoColumn(orderedIds, store.positions, {
        maxSize: store.config.value.max_size,
        gridSize: store.config.value.grid_size,
        spacing,
    });

    router.put(
        MapSelection.update().url,
        {
            map_solarsystems: [...positions.entries()].map(([id, position]) => ({
                id,
                position_x: Math.round(position.x),
                position_y: Math.round(position.y),
            })),
        },
        {
            preserveState: true,
            preserveScroll: true,
            onSuccess: () => store.clearSelection(),
            // The originator refreshes the map prop on success so the result is
            // visible even without a websocket; collaborators get the payload
            // event, and reconcileMap keeps the reload a cheap keyed diff.
            only: ['map'],
            onError: () => router.reload({ only: ['map'] }),
        },
    );
}
