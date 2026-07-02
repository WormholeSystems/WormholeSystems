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
            // "Reload nothing": Inertia treats an empty `only` array as a full
            // reload, so actions with no per-user props left to refresh request
            // the always-cheap 'errors' prop instead. The map itself is patched
            // by the sync layer from broadcast events, never reloaded here.
            only: ['errors'],
            onError: () => router.reload({ only: ['map'] }),
        },
    );
}
