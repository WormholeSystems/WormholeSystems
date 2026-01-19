import { convertEveScoutConnections, convertMapConnectionsToWorkerEdges } from '@/composables/routing/utils';
import { useEveScoutConnections } from '@/composables/useEveScoutConnections';
import { useMapUserSettings } from '@/composables/useMapUserSettings';
import { getRoutingWorkerClient } from '@/composables/useRoutingWorker';
import type { TMapConnection, TMapSolarsystem } from '@/pages/maps';
import type { WorkerClosestSystem } from '@/routing/types';
import type { MaybeRefOrGetter } from 'vue';
import { computed, ref, toValue, watch } from 'vue';

type UseClosestSystemsParams = {
    fromId: MaybeRefOrGetter<number | null>;
    condition: MaybeRefOrGetter<string>;
    limit: MaybeRefOrGetter<number>;
    ignoredSystems: MaybeRefOrGetter<number[]>;
    mapConnections: MaybeRefOrGetter<TMapConnection[]>;
    mapSolarsystems: MaybeRefOrGetter<TMapSolarsystem[]>;
};

export function useClosestSystemsCalculator(params: UseClosestSystemsParams) {
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

    const results = ref<WorkerClosestSystem[]>([]);
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
                const worker = await getRoutingWorkerClient();
                const ignored = [...(toValue(params.ignoredSystems) ?? [])];
                const settings = { ...routingSettings.value };
                const dynamicConnections = convertedConnections.value.map((edge) => ({ ...edge }));
                const eveScoutConnections = convertedEveScoutConnections.value.map((edge) => ({ ...edge }));
                const requests = [{ id: `${activeRequest}-closest`, type: 'closest' as const, fromId: from, condition, limit }];

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

                const response = responses.find((item) => item.type === 'closest');
                results.value = response?.results ?? [];
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
