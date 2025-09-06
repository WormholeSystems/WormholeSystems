import MapConnections from '@/routes/map-connections';
import { TMapConnection, TMassStatus, TShipSize } from '@/types/models';
import { router } from '@inertiajs/vue3';

export function updateMapConnection(
    map_connection: TMapConnection,
    data: {
        mass_status?: TMassStatus | string;
        ship_size?: TShipSize | string;
        marked_as_eol_at?: string | null | Date;
    },
) {
    return router.put(MapConnections.update(map_connection.id).url, data, {
        preserveScroll: true,
        preserveState: true,
        only: ['map', 'map_route_solarsystems', 'selected_map_solarsystem'],
    });
}
