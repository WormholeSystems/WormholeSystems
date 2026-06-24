import { usePath } from '@/composables/usePath';
import { useRallyRoute } from '@/composables/useRallyRoute';
import { TMapConnection } from '@/pages/maps';
import { TDataMapSolarSystem, TProcessedConnection } from '../types';

export function getConnectionWithSourceAndTarget(connection: TMapConnection, systemsById: Map<number, TDataMapSolarSystem>): TProcessedConnection {
    const { path } = usePath();
    const { getRallyRouteInfo } = useRallyRoute();

    const source = systemsById.get(connection.from_map_solarsystem_id);
    const target = systemsById.get(connection.to_map_solarsystem_id);

    const from_index = path.value?.findIndex((s) => s.id === source?.solarsystem_id) ?? -1;
    const to_index = path.value?.findIndex((s) => s.id === target?.solarsystem_id) ?? -1;
    const is_on_route = from_index !== -1 && to_index !== -1 && Math.abs(from_index - to_index) === 1;
    const rallyInfo = source && target ? getRallyRouteInfo(source.solarsystem_id, target.solarsystem_id) : { onRoute: false, reversed: false };

    return {
        ...connection,
        source: source!,
        target: target!,
        is_on_route,
        is_on_rally_route: rallyInfo.onRoute,
        rally_route_reversed: rallyInfo.reversed,
    };
}
