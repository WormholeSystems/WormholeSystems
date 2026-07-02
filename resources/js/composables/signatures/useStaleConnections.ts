import { useMap } from '@/composables/useMap';
import { cleanStaleMapConnections } from '@/map/api';
import type { TMapSolarsystem } from '@/pages/maps';
import { computed } from 'vue';

const STALE_THRESHOLD_MS = 60 * 60 * 1000;

export type TStaleConnection = {
    id: number;
    from: string;
    to: string;
};

export function useStaleConnections() {
    const map = useMap();

    const systems_by_id = computed(() => {
        return (map.value.map_solarsystems ?? []).reduce(
            (acc, system) => {
                acc[system.id] = system;
                return acc;
            },
            {} as Record<number, TMapSolarsystem>,
        );
    });

    function systemName(id: number): string {
        const system = systems_by_id.value[id];
        return system?.alias ?? system?.solarsystem?.name ?? 'Unknown';
    }

    const stale_connections = computed<TStaleConnection[]>(() => {
        const now = Date.now();

        return (map.value.map_connections ?? [])
            .filter((connection) => {
                if (connection.lifetime_status !== 'critical' || !connection.lifetime_status_updated_at) {
                    return false;
                }
                return now - Date.parse(connection.lifetime_status_updated_at) > STALE_THRESHOLD_MS;
            })
            .map((connection) => ({
                id: connection.id,
                from: systemName(connection.from_map_solarsystem_id),
                to: systemName(connection.to_map_solarsystem_id),
            }));
    });

    function cleanMap() {
        return cleanStaleMapConnections(map.value);
    }

    return {
        stale_connections,
        cleanMap,
    };
}
