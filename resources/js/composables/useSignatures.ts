import { TProcessedConnection } from '@/composables/map';
import { useMap } from '@/composables/useMap';
import { useSelectedMapSolarsystem } from '@/composables/useSelectedMapSolarsystem';
import signature_tree from '@/const/signatures';
import { TMapSolarSystem } from '@/types/models';
import { computed } from 'vue';

export function useSignatures() {
    const map = useMap();
    const selected_map_solarsystem = useSelectedMapSolarsystem();

    const solarsystem_class = computed(() => selected_map_solarsystem.value?.class);
    const solarsystem_security = computed(() => selected_map_solarsystem.value?.solarsystem?.security);

    const relevant_signatures = computed(() => {
        if (solarsystem_class.value) return getWormholeSignatures(solarsystem_class.value);
        return getKnownSpaceSignatures(solarsystem_security.value!, selected_map_solarsystem.value!);
    });

    const map_solarsystems = computed(() => {
        return map.value.map_solarsystems?.reduce(
            (acc, system) => {
                acc[system.id] = system;
                return acc;
            },
            {} as Record<number, TMapSolarSystem>,
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
        relevant_signatures,
        map_solarsystems,
        connections,
    };
}

function getWormholeSignatures(solarsystem_class: number) {
    return signature_tree.wormhole_space[solarsystem_class as keyof typeof signature_tree.wormhole_space] || [];
}

function getKnownSpaceSignatures(solarsystem_security: number, map_solarsystem: TMapSolarSystem) {
    if (solarsystem_security >= 0.5) return signature_tree.known_space.hs;
    if (solarsystem_security >= 0.1) return signature_tree.known_space.ls;
    if (map_solarsystem.solarsystem?.region?.name === 'Pochven') return signature_tree.known_space.pv;
    return signature_tree.known_space.ls;
}
