import { useRoutingSetup } from '@/composables/routing/useRoutingSetup';
import { findRoute, initializeRouting } from '@/composables/useRoutingWorker';
import type { TMapConnection, TMapSolarsystem } from '@/pages/maps';
import type { RouteStep } from '@/routing/types';
import type { MaybeRefOrGetter } from 'vue';
import { ref, toValue, watch } from 'vue';

type UseRouteCalculatorParams = {
    fromId: MaybeRefOrGetter<number | null>;
    toId: MaybeRefOrGetter<number | null>;
    ignoredSystems: MaybeRefOrGetter<number[]>;
    mapConnections: MaybeRefOrGetter<TMapConnection[]>;
    mapSolarsystems: MaybeRefOrGetter<TMapSolarsystem[]>;
};

export function useRouteCalculator(params: UseRouteCalculatorParams) {
    const { routingSettings, convertedConnections, convertedEveScoutConnections, getConnections } = useRoutingSetup({
        mapConnections: params.mapConnections,
        mapSolarsystems: params.mapSolarsystems,
    });

    const route = ref<RouteStep[]>([]);
    const jumps = ref(0);
    const cost = ref(0);
    const isLoading = ref(false);

    let requestCounter = 0;

    watch(
        [
            () => toValue(params.fromId),
            () => toValue(params.toId),
            () => toValue(params.ignoredSystems),
            () => convertedConnections.value,
            () => convertedEveScoutConnections.value,
            () => routingSettings.value,
        ],
        async () => {
            const activeRequest = ++requestCounter;
            const from = toValue(params.fromId);
            const to = toValue(params.toId);

            if (!from || !to || from === to) {
                route.value = [];
                jumps.value = 0;
                cost.value = 0;
                isLoading.value = false;
                return;
            }

            isLoading.value = true;

            try {
                await initializeRouting();

                const { dynamicConnections, eveScoutConnections } = getConnections();
                const result = findRoute({ ...routingSettings.value }, from, to, dynamicConnections, eveScoutConnections, [
                    ...(toValue(params.ignoredSystems) ?? []),
                ]);

                if (activeRequest !== requestCounter) {
                    return;
                }

                route.value = result.route;
                jumps.value = result.jumps;
                cost.value = result.cost;
            } catch {
                route.value = [];
                jumps.value = 0;
                cost.value = 0;
            } finally {
                if (activeRequest === requestCounter) {
                    isLoading.value = false;
                }
            }
        },
        { immediate: true, deep: true },
    );

    return {
        route,
        jumps,
        cost,
        isLoading,
        routingSettings,
    };
}
