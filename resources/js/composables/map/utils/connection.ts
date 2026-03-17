import { usePath } from '@/composables/usePath';
import { TMapConnection } from '@/pages/maps';
import { mapState } from '../state';
import { TProcessedConnection } from '../types';

export function getConnectionWithSourceAndTarget(connection: TMapConnection): TProcessedConnection {
    const { path } = usePath();

    const source = mapState.map_solarsystems.find((s) => s.id === connection.from_map_solarsystem_id)!;
    const target = mapState.map_solarsystems.find((s) => s.id === connection.to_map_solarsystem_id)!;

    const from_map_solarsystem = mapState.map_solarsystems.find((s) => s.id === connection.from_map_solarsystem_id);
    const to_map_solarsystem = mapState.map_solarsystems.find((s) => s.id === connection.to_map_solarsystem_id);
    const from_index = path.value?.findIndex((s) => s.id === from_map_solarsystem?.solarsystem_id) ?? -1;
    const to_index = path.value?.findIndex((s) => s.id === to_map_solarsystem?.solarsystem_id) ?? -1;
    const is_on_route = from_index !== -1 && to_index !== -1 && Math.abs(from_index - to_index) === 1;

    return {
        ...connection,
        source,
        target,
        is_on_route,
    };
}
