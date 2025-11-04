import { useMap } from '@/composables/useMap';
import { useSelectedMapSolarsystem } from '@/composables/useSelectedMapSolarsystem';
import { TMapSolarsystem } from '@/pages/maps';
import { computed } from 'vue';
import { TProcessedConnection } from '../types';

export function useSignatures() {
    const map = useMap();
    const selected_map_solarsystem = useSelectedMapSolarsystem();

    const solarsystem_class = computed(() => selected_map_solarsystem.value?.solarsystem.class);
    const solarsystem_security = computed(() => selected_map_solarsystem.value?.solarsystem?.security);

    const map_solarsystems = computed(() => {
        return map.value.map_solarsystems?.reduce(
            (acc, system) => {
                acc[system.id] = system;
                return acc;
            },
            {} as Record<number, TMapSolarsystem>,
        );
    });

    const connections = computed(() => {
        return selected_map_solarsystem
            .value!.map_connections!.map((connection) => {
                const other_id =
                    connection.from_map_solarsystem_id === selected_map_solarsystem.value?.id
                        ? connection.to_map_solarsystem_id
                        : connection.from_map_solarsystem_id;
                const other_system = map_solarsystems.value![other_id];

                if (!other_system) {
                    throw new Error(`Connection to unknown system with ID ${other_id}`);
                }

                return {
                    ...connection,
                    source: selected_map_solarsystem.value!,
                    target: other_system,
                } satisfies TProcessedConnection;
            })
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
