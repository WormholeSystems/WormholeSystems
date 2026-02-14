import { useRoutingSetup } from '@/composables/routing/useRoutingSetup';
import { findClosestSystems, initializeRouting } from '@/composables/useRoutingWorker';
import type { TMapConnection, TMapSolarsystem } from '@/pages/maps';
import type { ClosestSystem } from '@/routing/types';
import type { MaybeRefOrGetter } from 'vue';
import { ref, toValue, watch } from 'vue';

type UseClosestSystemsParams = {
    fromId: MaybeRefOrGetter<number | null>;
    condition: MaybeRefOrGetter<string>;
    limit: MaybeRefOrGetter<number>;
    ignoredSystems: MaybeRefOrGetter<number[]>;
    mapConnections: MaybeRefOrGetter<TMapConnection[]>;
    mapSolarsystems: MaybeRefOrGetter<TMapSolarsystem[]>;
};

export function useClosestSystemsCalculator(params: UseClosestSystemsParams) {
    const { routingSettings, convertedConnections, convertedEveScoutConnections, getConnections } = useRoutingSetup({
        mapConnections: params.mapConnections,
        mapSolarsystems: params.mapSolarsystems,
    });

    const results = ref<ClosestSystem[]>([]);
    const isLoading = ref(false);

    let requestCounter = 0;

    watch(
        [
            () => toValue(params.fromId),
            () => toValue(params.condition),
            () => toValue(params.limit),
            () => toValue(params.ignoredSystems),
            () => convertedConnections.value,
            () => convertedEveScoutConnections.value,
            () => routingSettings.value,
        ],
        async () => {
            const activeRequest = ++requestCounter;
            const from = toValue(params.fromId);
            const condition = toValue(params.condition) || 'observatories';
            const limit = Math.max(1, toValue(params.limit) ?? 15);

            if (!from) {
                results.value = [];
                isLoading.value = false;
                return;
            }

            isLoading.value = true;

            try {
                await initializeRouting();

                const { dynamicConnections, eveScoutConnections } = getConnections();
                const closestResults = findClosestSystems(
                    { ...routingSettings.value },
                    from,
                    condition,
                    limit,
                    dynamicConnections,
                    eveScoutConnections,
                    [...(toValue(params.ignoredSystems) ?? [])],
                );

                if (activeRequest !== requestCounter) {
                    return;
                }

                results.value = closestResults;
            } catch {
                results.value = [];
            } finally {
                if (activeRequest === requestCounter) {
                    isLoading.value = false;
                }
            }
        },
        { immediate: true, deep: true },
    );

    return {
        results,
        isLoading,
    };
}
