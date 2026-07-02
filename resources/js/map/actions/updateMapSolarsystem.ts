import type { TMapSolarsystem, TShowMapProps } from '@/pages/maps';
import MapSolarsystems from '@/routes/map-solarsystems';
import type { AppPageProps } from '@/types';
import { router, usePage } from '@inertiajs/vue3';

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
): void {
    const page = usePage<AppPageProps<TShowMapProps>>();
    const isDetailPanelTarget = page.props.selected_map_solarsystem?.id === map_solarsystem.id;

    return router.put(MapSolarsystems.update(map_solarsystem.id).url, data, {
        preserveState: true,
        preserveScroll: true,
        only: isDetailPanelTarget ? ['selected_map_solarsystem'] : ['errors'],
        onError: () => router.reload({ only: ['map'] }),
    });
}
