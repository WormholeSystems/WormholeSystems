import type { TStaticConnections, TStaticConstellation, TStaticData, TStaticRegion, TStaticSolarsystem } from '@/types/static-data';
import type { MaybeRefOrGetter } from 'vue';
import { computed, shallowReadonly, shallowRef, toValue } from 'vue';

import connections from '../../static/connections.json';
import constellations from '../../static/constellations.json';
import regions from '../../static/regions.json';
import solarsystems from '../../static/solarsystems.json';

const staticData = shallowRef<TStaticData | null>(null);
let loadPromise: Promise<TStaticData> | null = null;

async function loadStaticData(): Promise<TStaticData> {
    if (staticData.value) {
        return staticData.value;
    }

    if (!loadPromise) {
        loadPromise = Promise.resolve().then(() => {
            const regionsData = regions as TStaticRegion[];
            const constellationsData = constellations as TStaticConstellation[];
            const solarsystemsData = solarsystems as TStaticSolarsystem[];
            const connectionsData = connections as TStaticConnections;

            const normalizedSolarsystems = solarsystemsData.map((solarsystem) => ({
                ...solarsystem,
                sovereignty: null,
                connection_type: solarsystem.connection_type ?? null,
            })) satisfies TStaticSolarsystem[];

            const payload = {
                regions: regionsData,
                constellations: constellationsData,
                solarsystems: normalizedSolarsystems,
                connections: connectionsData,
            } satisfies TStaticData;

            staticData.value = payload;

            return payload;
        });
    }

    return loadPromise;
}

export function useStaticData() {
    return {
        staticData: shallowReadonly(staticData),
        loadStaticData,
    };
}

export async function preloadStaticData(): Promise<void> {
    await loadStaticData();
}

export function useStaticRegion(regionId: MaybeRefOrGetter<number | null | undefined>) {
    const { staticData, loadStaticData } = useStaticData();

    void loadStaticData();

    return computed(() => {
        const id = toValue(regionId);
        if (!id) {
            return null;
        }

        return staticData.value?.regions.find((region) => region.id === id) ?? null;
    });
}

export function useStaticConstellation(constellationId: MaybeRefOrGetter<number | null | undefined>) {
    const { staticData, loadStaticData } = useStaticData();

    void loadStaticData();

    return computed(() => {
        const id = toValue(constellationId);
        if (!id) {
            return null;
        }

        return staticData.value?.constellations.find((constellation) => constellation.id === id) ?? null;
    });
}
