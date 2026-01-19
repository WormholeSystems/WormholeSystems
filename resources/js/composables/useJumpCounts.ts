import { convertEveScoutConnections, convertMapConnectionsToWorkerEdges } from '@/composables/routing/utils';
import { useEveScoutConnections } from '@/composables/useEveScoutConnections';
import { useMapUserSettings } from '@/composables/useMapUserSettings';
import { getRoutingWorkerClient } from '@/composables/useRoutingWorker';
import type { TMapConnection, TMapSolarsystem } from '@/pages/maps';
import type { WorkerRouteResult } from '@/routing/types';
import type { MaybeRefOrGetter } from 'vue';
import { computed, ref, toValue, watch } from 'vue';

type UseJumpCountsParams = {
    fromId: MaybeRefOrGetter<number | null>;
    targets: MaybeRefOrGetter<number[]>;
    mapConnections: MaybeRefOrGetter<TMapConnection[]>;
    mapSolarsystems: MaybeRefOrGetter<TMapSolarsystem[]>;
    ignoredSystems: MaybeRefOrGetter<number[]>;
};

export function useJumpCounts(params: UseJumpCountsParams) {
    const mapUserSettings = useMapUserSettings();
    const eveScoutConnections = useEveScoutConnections();

    const jumpsByTarget = ref(new Map<number, number>());
    const routesByTarget = ref(new Map<number, WorkerRouteResult>());
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
            () => toValue(params.targets),
            () => toValue(params.mapConnections),
            () => toValue(params.mapSolarsystems),
            () => toValue(params.ignoredSystems),
            () => routingSettings.value,
        ],
        async () => {
            const fromId = toValue(params.fromId);
            const targets = toValue(params.targets) ?? [];

            if (!fromId || targets.length === 0) {
                jumpsByTarget.value = new Map();
                routesByTarget.value = new Map();
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
                const requests = targets.map((targetId, index) => ({
                    id: `${index}-${targetId}`,
                    type: 'route' as const,
                    fromId,
                    toId: targetId,
                }));

                const responses = await worker.compute({
                    settings,
                    dynamicConnections,
                    eveScoutConnections: scoutConnections,
                    ignoredSystems: ignored,
                    requests,
                });

                const jumpsMap = new Map<number, number>();
                const routesMap = new Map<number, WorkerRouteResult>();
                for (const response of responses) {
                    if (response.type !== 'route') {
                        continue;
                    }

                    const targetId = parseInt(response.id.split('-').pop() ?? '', 10);
                    if (!Number.isNaN(targetId)) {
                        jumpsMap.set(targetId, response.jumps);
                        routesMap.set(targetId, response);
                    }
                }

                jumpsByTarget.value = jumpsMap;
                routesByTarget.value = routesMap;
            } finally {
                isLoading.value = false;
            }
        },
        { immediate: true, deep: true },
    );

    return {
        jumpsByTarget,
        routesByTarget,
        isLoading,
    };
}
