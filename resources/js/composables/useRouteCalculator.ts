import { convertEveScoutConnections, convertMapConnectionsToWorkerEdges } from '@/composables/routing/utils';
import { useEveScoutConnections } from '@/composables/useEveScoutConnections';
import { useMapUserSettings } from '@/composables/useMapUserSettings';
import { getRoutingWorkerClient } from '@/composables/useRoutingWorker';
import type { TMapConnection, TMapSolarsystem } from '@/pages/maps';
import type { WorkerRouteStep } from '@/routing/types';
import type { MaybeRefOrGetter } from 'vue';
import { computed, ref, toValue, watch } from 'vue';

type UseRouteCalculatorParams = {
    fromId: MaybeRefOrGetter<number | null>;
    toId: MaybeRefOrGetter<number | null>;
    ignoredSystems: MaybeRefOrGetter<number[]>;
    mapConnections: MaybeRefOrGetter<TMapConnection[]>;
    mapSolarsystems: MaybeRefOrGetter<TMapSolarsystem[]>;
};

export function useRouteCalculator(params: UseRouteCalculatorParams) {
    const mapUserSettings = useMapUserSettings();
    const eveScoutConnections = useEveScoutConnections();

    const convertedConnections = computed(() => convertMapConnectionsToWorkerEdges(toValue(params.mapConnections), toValue(params.mapSolarsystems)));

    const convertedEveScoutConnections = computed(() =>
        convertEveScoutConnections(eveScoutConnections.value, mapUserSettings.value.route_use_evescout),
    );

    const routingSettings = computed(() => ({
        routePreference: mapUserSettings.value.route_preference,
        securityPenalty: mapUserSettings.value.security_penalty,
        allowEol: mapUserSettings.value.route_allow_eol,
        massStatus: mapUserSettings.value.route_allow_mass_status,
        useEveScout: mapUserSettings.value.route_use_evescout,
    }));

    const route = ref<WorkerRouteStep[]>([]);
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
                const worker = await getRoutingWorkerClient();
                const ignored = [...(toValue(params.ignoredSystems) ?? [])];
                const settings = { ...routingSettings.value };
                const dynamicConnections = convertedConnections.value.map((edge) => ({ ...edge }));
                const eveScoutConnections = convertedEveScoutConnections.value.map((edge) => ({ ...edge }));
                const requests = [{ id: `${activeRequest}-route`, type: 'route' as const, fromId: from, toId: to }];

                const responses = await worker.compute({
                    settings,
                    dynamicConnections,
                    eveScoutConnections,
                    ignoredSystems: ignored,
                    requests,
                });

                if (activeRequest !== requestCounter) {
                    return;
                }

                const response = responses.find((item) => item.type === 'route');
                if (!response) {
                    route.value = [];
                    jumps.value = 0;
                    cost.value = 0;
                    return;
                }

                route.value = response.route;
                jumps.value = response.jumps;
                cost.value = response.cost;
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
