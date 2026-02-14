import { useRoutingSetup } from '@/composables/routing/useRoutingSetup';
import { findRoute, initializeRouting } from '@/composables/useRoutingWorker';
import type { TMapConnection, TMapSolarsystem } from '@/pages/maps';
import type { RouteResult } from '@/routing/types';
import type { MaybeRefOrGetter } from 'vue';
import { ref, toValue, watch } from 'vue';

type UseJumpCountsParams = {
    fromId: MaybeRefOrGetter<number | null>;
    targets: MaybeRefOrGetter<number[]>;
    mapConnections: MaybeRefOrGetter<TMapConnection[]>;
    mapSolarsystems: MaybeRefOrGetter<TMapSolarsystem[]>;
    ignoredSystems: MaybeRefOrGetter<number[]>;
    includeEveScout?: MaybeRefOrGetter<boolean>;
};

export function useJumpCounts(params: UseJumpCountsParams) {
    const { routingSettings, convertedEveScoutConnections, getConnections } = useRoutingSetup({
        mapConnections: params.mapConnections,
        mapSolarsystems: params.mapSolarsystems,
        includeEveScout: params.includeEveScout,
    });

    const jumpsByTarget = ref(new Map<number, number>());
    const routesByTarget = ref(new Map<number, RouteResult>());
    const isLoading = ref(false);

    watch(
        [
            () => toValue(params.fromId),
            () => toValue(params.targets),
            () => toValue(params.mapConnections),
            () => toValue(params.mapSolarsystems),
            () => toValue(params.ignoredSystems),
            () => routingSettings.value,
            () => convertedEveScoutConnections.value,
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
                await initializeRouting();

                const { dynamicConnections, eveScoutConnections } = getConnections();
                const settings = { ...routingSettings.value };
                const ignored = [...(toValue(params.ignoredSystems) ?? [])];

                const jumpsMap = new Map<number, number>();
                const routesMap = new Map<number, RouteResult>();
                for (const targetId of targets) {
                    const result = findRoute(settings, fromId, targetId, dynamicConnections, eveScoutConnections, ignored);
                    if (result.route.length === 0) {
                        continue;
                    }
                    jumpsMap.set(targetId, result.jumps);
                    routesMap.set(targetId, result);
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
