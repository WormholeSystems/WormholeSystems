import { TMapConnection } from '@/pages/maps';
import MapConnections from '@/routes/map-connections';
import { router } from '@inertiajs/vue3';

export function deleteMapConnection(map_connection: TMapConnection) {
    return router.delete(MapConnections.destroy(map_connection.id).url, {
        preserveScroll: true,
        preserveState: true,
        only: ['map', 'map_route_solarsystems'],
    });
}
