import { index } from '@/routes/api/sovereignties';
import type { TSovereignty } from '@/types/models';
import type { MaybeRefOrGetter } from 'vue';
import { computed, readonly, shallowRef, toValue } from 'vue';

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
        loadPromise = fetch(index.url(), { cache: 'force-cache' })
            .then((response) => {
                if (!response.ok) {
                    throw new Error(`Failed to fetch sovereignty data: ${response.status}`);
                }
                return response.json() as Promise<Record<number, TSovereignty>>;
            })
            .then((data) => {
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
