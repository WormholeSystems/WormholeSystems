import { convertEveScoutConnections, convertMapConnectionsToWorkerEdges } from '@/composables/routing/utils';
import { useEveScoutConnections } from '@/composables/useEveScoutConnections';
import { useMapUserSettings } from '@/composables/useMapUserSettings';
import { getRoutingWorkerClient } from '@/composables/useRoutingWorker';
import type { TMapConnection, TMapSolarsystem, TResolvedMapRouteSolarsystem } from '@/pages/maps';
import type { WorkerRouteResult } from '@/routing/types';
import type { MaybeRefOrGetter } from 'vue';
import { computed, ref, toValue, watch } from 'vue';

type UseDestinationRoutesParams = {
    fromId: MaybeRefOrGetter<number | null>;
    destinations: MaybeRefOrGetter<TResolvedMapRouteSolarsystem[]>;
    mapConnections: MaybeRefOrGetter<TMapConnection[]>;
    mapSolarsystems: MaybeRefOrGetter<TMapSolarsystem[]>;
    ignoredSystems: MaybeRefOrGetter<number[]>;
};

export function useDestinationRoutes(params: UseDestinationRoutesParams) {
    const mapUserSettings = useMapUserSettings();
    const eveScoutConnections = useEveScoutConnections();

    const routesByDestination = ref(new Map<number, WorkerRouteResult>());
    const isLoading = ref(false);

    const routingSettings = computed(() => ({
        routePreference: mapUserSettings.value.route_preference,
        securityPenalty: mapUserSettings.value.security_penalty,
        allowEol: mapUserSettings.value.route_allow_eol,
        massStatus: mapUserSettings.value.route_allow_mass_status,
        useEveScout: mapUserSettings.value.route_use_evescout,
    }));

    watch(
        [
            () => toValue(params.fromId),
            () => toValue(params.destinations),
            () => toValue(params.mapConnections),
            () => toValue(params.mapSolarsystems),
            () => toValue(params.ignoredSystems),
            () => routingSettings.value,
            () => eveScoutConnections.value,
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
                const worker = await getRoutingWorkerClient();
                const dynamicConnections = convertMapConnectionsToWorkerEdges(toValue(params.mapConnections), toValue(params.mapSolarsystems)).map(
                    (edge) => ({ ...edge }),
                );
                const scoutConnections = convertEveScoutConnections(eveScoutConnections.value, mapUserSettings.value.route_use_evescout).map(
                    (edge) => ({ ...edge }),
                );
                const ignored = [...(toValue(params.ignoredSystems) ?? [])];
                const settings = { ...routingSettings.value };
                const requests = destinations.map((destination) => ({
                    id: `${destination.id}-route`,
                    type: 'route' as const,
                    fromId,
                    toId: destination.solarsystem_id,
                }));

                const responses = await worker.compute({
                    settings,
                    dynamicConnections,
                    eveScoutConnections: scoutConnections,
                    ignoredSystems: ignored,
                    requests,
                });

                const routeMap = new Map<number, WorkerRouteResult>();
                for (const response of responses) {
                    if (response.type !== 'route') {
                        continue;
                    }

                    const destinationId = Number(response.id.split('-')[0]);
                    if (!Number.isNaN(destinationId)) {
                        routeMap.set(destinationId, response);
                    }
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
