import { useRoutingSetup } from '@/composables/routing/useRoutingSetup';
import { useMap } from '@/composables/useMap';
import { findRoute, initializeRouting } from '@/composables/useRoutingWorker';
import type { RouteStep } from '@/routing/types';
import { readonly, ref, watch } from 'vue';

/** The route from home to each rally point, keyed by the rally point's solarsystem id. */
const rallyRoutes = ref<Record<number, RouteStep[]>>({});
let initialized = false;

export function useRallyRoute() {
    if (!initialized) {
        initialized = true;

        const map = useMap();

        const { routingSettings, convertedEveScoutConnections, getConnections } = useRoutingSetup({
            mapConnections: () => map.value.map_connections ?? [],
            mapSolarsystems: () => map.value.map_solarsystems ?? [],
        });

        watch(
            [
                () => map.value.home_solarsystem_id,
                () => map.value.rally_solarsystem_ids,
                () => map.value.map_connections,
                () => map.value.map_solarsystems,
                routingSettings,
                convertedEveScoutConnections,
            ],
            async () => {
                const homeId = map.value.home_solarsystem_id;
                const rallyIds = map.value.rally_solarsystem_ids;

                if (!homeId || rallyIds.length === 0) {
                    rallyRoutes.value = {};
                    return;
                }

                await initializeRouting();

                const { dynamicConnections, eveScoutConnections } = getConnections();

                const results = await Promise.all(
                    rallyIds.map((rallyId) => findRoute(routingSettings.value, homeId, rallyId, dynamicConnections, eveScoutConnections, [])),
                );

                rallyRoutes.value = Object.fromEntries(rallyIds.map((rallyId, index) => [rallyId, results[index].route]));
            },
            { immediate: true },
        );
    }

    function getRallyRouteInfo(fromSolarsystemId: number, toSolarsystemId: number): { onRoute: boolean; reversed: boolean } {
        for (const route of Object.values(rallyRoutes.value)) {
            for (let i = 0; i < route.length - 1; i++) {
                if (route[i].id === fromSolarsystemId && route[i + 1].id === toSolarsystemId) {
                    return { onRoute: true, reversed: false };
                }
                if (route[i].id === toSolarsystemId && route[i + 1].id === fromSolarsystemId) {
                    return { onRoute: true, reversed: true };
                }
            }
        }

        return { onRoute: false, reversed: false };
    }

    return {
        rallyRoutes: readonly(rallyRoutes),
        getRallyRouteInfo,
    };
}
