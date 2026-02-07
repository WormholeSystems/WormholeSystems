import { fetchCompressedJson } from '@/lib/compressedJson';
import type { TSovereignty } from '@/types/models';
import type { MaybeRefOrGetter } from 'vue';
import { computed, readonly, shallowRef, toValue } from 'vue';

const sovereigntyUrl = '/static/sovereignty.json.gz';

const sovereigntyMap = shallowRef<Record<number, TSovereignty> | null>(null);
let loadPromise: Promise<Record<number, TSovereignty>> | null = null;

async function loadSovereigntyData(): Promise<Record<number, TSovereignty>> {
    if (typeof window === 'undefined') {
        return {};
    }

    if (sovereigntyMap.value) {
        return sovereigntyMap.value;
    }

    if (!loadPromise) {
        loadPromise = fetchCompressedJson<Record<number, TSovereignty>>(sovereigntyUrl).then((data) => {
            sovereigntyMap.value = data;
            return data;
        });
    }

    return loadPromise;
}

export function useSovereigntyData() {
    return {
        sovereignty: readonly(sovereigntyMap),
        loadSovereigntyData,
        getSovereignty: (solarsystemId: number): TSovereignty | null => sovereigntyMap.value?.[solarsystemId] ?? null,
    };
}

export async function preloadSovereigntyData(): Promise<void> {
    await loadSovereigntyData();
}

export function useSovereignty(solarsystemId: MaybeRefOrGetter<number | null | undefined>) {
    const { sovereignty, loadSovereigntyData } = useSovereigntyData();

    void loadSovereigntyData();

    return computed(() => {
        const id = toValue(solarsystemId);
        if (!id) {
            return null;
        }

        return sovereignty.value?.[id] ?? null;
    });
}
