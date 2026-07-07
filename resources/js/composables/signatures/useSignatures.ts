import { useSelectedMapSolarsystem } from '@/composables/useSelectedMapSolarsystem';
import type { TProcessedConnection } from '@/map/api';
import { useMapSolarsystems } from '@/map/api';
import { TMapSolarsystem } from '@/pages/maps';
import { computed } from 'vue';

export function useSignatures() {
    const selected_map_solarsystem = useSelectedMapSolarsystem();
    const { map_solarsystems: live_map_solarsystems } = useMapSolarsystems();

    const solarsystem_class = computed(() => selected_map_solarsystem.value?.solarsystem.class);
    const solarsystem_security = computed(() => selected_map_solarsystem.value?.solarsystem?.security);

    /**
     * Keyed off the realtime-patched store, not the `map` page prop: broadcast events
     * upsert new systems into the store only, so the prop can be stale when the selected
     * system's connections are reloaded and already reference a freshly added system.
     */
    const map_solarsystems = computed(() => {
        return new Map<number, TMapSolarsystem>(live_map_solarsystems.value.map((system) => [system.id, system]));
    });

    const connections = computed(() => {
        const selected = selected_map_solarsystem.value;

        if (!selected?.map_connections) {
            return [];
        }

        return selected.map_connections
            .map((connection): TProcessedConnection | null => {
                const other_id =
                    connection.from_map_solarsystem_id === selected.id ? connection.to_map_solarsystem_id : connection.from_map_solarsystem_id;
                const other_system = map_solarsystems.value.get(other_id);

                if (!other_system) {
                    return null;
                }

                return {
                    ...connection,
                    source: selected,
                    target: other_system,
                };
            })
            .filter((connection): connection is TProcessedConnection => connection !== null)
            .toSorted((a, b) => {
                if (a.target.alias && b.target.alias) {
                    return a.target.alias.localeCompare(b.target.alias);
                }
                if (a.target.alias) return -1;
                if (b.target.alias) return 1;

                return a.target.solarsystem!.name.localeCompare(b.target.solarsystem!.name);
            });
    });

    return {
        solarsystem_class,
        solarsystem_security,
        map_solarsystems,
        connections,
    };
}
