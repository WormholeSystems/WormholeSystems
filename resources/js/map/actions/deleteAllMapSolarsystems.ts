import { useMapStore } from '@/map/store/mapStore';
import MapSelection from '@/routes/map-selection';
import { router } from '@inertiajs/vue3';

export function deleteAllMapSolarsystems(): void {
    const store = useMapStore();

    router.delete(MapSelection.destroy().url, {
        data: {
            map_solarsystem_ids: [...store.systems.values()].filter((system) => !system.pinned).map((system) => system.id),
        },
        preserveState: true,
        preserveScroll: true,
        only: ['map', 'map_navigation', 'selected_map_solarsystem'],
        onError: () => router.reload({ only: ['map'] }),
    });
}
