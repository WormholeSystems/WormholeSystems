import { useRoutingSetup } from '@/composables/routing/useRoutingSetup';
import { findRoute, initializeRouting } from '@/composables/useRoutingWorker';
import type { TMapConnection, TMapSolarsystem, TResolvedMapRouteSolarsystem } from '@/pages/maps';
import type { RouteResult } from '@/routing/types';
import type { MaybeRefOrGetter } from 'vue';
import { ref, toValue, watch } from 'vue';

type UseDestinationRoutesParams = {
    fromId: MaybeRefOrGetter<number | null>;
    destinations: MaybeRefOrGetter<TResolvedMapRouteSolarsystem[]>;
    mapConnections: MaybeRefOrGetter<TMapConnection[]>;
    mapSolarsystems: MaybeRefOrGetter<TMapSolarsystem[]>;
    ignoredSystems: MaybeRefOrGetter<number[]>;
};

export function useDestinationRoutes(params: UseDestinationRoutesParams) {
    const { routingSettings, convertedEveScoutConnections, getConnections } = useRoutingSetup({
        mapConnections: params.mapConnections,
        mapSolarsystems: params.mapSolarsystems,
    });

    const routesByDestination = ref(new Map<number, RouteResult>());
    const isLoading = ref(false);

    watch(
        [
            () => toValue(params.fromId),
            () => toValue(params.destinations),
            () => toValue(params.mapConnections),
            () => toValue(params.mapSolarsystems),
            () => toValue(params.ignoredSystems),
            () => routingSettings.value,
            () => convertedEveScoutConnections.value,
        ],
        async () => {
            const fromId = toValue(params.fromId);
            const destinations = toValue(params.destinations) ?? [];

            if (!fromId || destinations.length === 0) {
                routesByDestination.value = new Map();
                isLoading.value = false;
                return;
            }

            isLoading.value = true;

            try {
                await initializeRouting();

                const { dynamicConnections, eveScoutConnections } = getConnections();
                const settings = { ...routingSettings.value };
                const ignored = [...(toValue(params.ignoredSystems) ?? [])];

                const routeMap = new Map<number, RouteResult>();
                for (const destination of destinations) {
                    const result = findRoute(settings, fromId, destination.solarsystem_id, dynamicConnections, eveScoutConnections, ignored);
                    if (result.route.length === 0) {
                        continue;
                    }
                    routeMap.set(destination.id, result);
                }

                routesByDestination.value = routeMap;
            } finally {
                isLoading.value = false;
            }
        },
        { immediate: true, deep: true },
    );

    return {
        routesByDestination,
        isLoading,
    };
}
