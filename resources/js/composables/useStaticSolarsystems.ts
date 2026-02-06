import { useStaticData } from '@/composables/useStaticData';
import type { TStaticSolarsystem } from '@/types/static-data';
import type { MaybeRefOrGetter } from 'vue';
import { computed, toValue } from 'vue';

export function useStaticSolarsystems() {
    const { staticData, loadStaticData } = useStaticData();

    void loadStaticData();

    const solarsystemsById = computed(() => {
        const map = new Map<number, TStaticSolarsystem>();

        for (const solarsystem of staticData.value?.solarsystems ?? []) {
            map.set(solarsystem.id, solarsystem);
        }

        return map;
    });

    const getSolarsystemById = (solarsystemId?: number | null): TStaticSolarsystem | null => {
        if (!solarsystemId) {
            return null;
        }

        return solarsystemsById.value.get(solarsystemId) ?? null;
    };

    const fallbackSolarsystem = (id: number, name?: string): TStaticSolarsystem => ({
        id,
        name: name ?? 'Unknown',
        region_id: 0,
        constellation_id: 0,
        class: null,
        security: 0,
        type: 'wormhole',
        region: { id: 0, name: 'Unknown' },
        sovereignty: null,
        statics: null,
        effect: null,
        has_jove_observatory: false,
        has_stations: false,
        services: [],
        connection_type: null,
    });

    const resolveSolarsystem = (
        reference?: number | { id: number; name?: string; connection_type?: 'wormhole' | 'stargate' | null } | null,
    ): TStaticSolarsystem => {
        if (!reference) {
            return fallbackSolarsystem(0);
        }

        const id = typeof reference === 'number' ? reference : reference.id;
        const name = typeof reference === 'number' ? undefined : reference.name;
        const connection_type = typeof reference === 'number' ? null : reference.connection_type;

        if (!id) {
            return fallbackSolarsystem(0, name);
        }

        const staticSystem = getSolarsystemById(id);

        if (!staticSystem) {
            return {
                ...fallbackSolarsystem(id, name),
                connection_type: connection_type ?? null,
            };
        }

        return {
            ...staticSystem,
            name: name ?? staticSystem.name,
            connection_type: connection_type ?? staticSystem.connection_type,
        };
    };

    return {
        solarsystemsById,
        getSolarsystemById,
        resolveSolarsystem,
    };
}

export function useStaticSolarsystem(solarsystemId: MaybeRefOrGetter<number | null | undefined>) {
    const { getSolarsystemById } = useStaticSolarsystems();

    return computed(() => getSolarsystemById(toValue(solarsystemId)));
}
