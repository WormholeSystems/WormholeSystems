import { useRoutingSetup } from '@/composables/routing/useRoutingSetup';
import { useMap } from '@/composables/useMap';
import { findRoute, initializeRouting } from '@/composables/useRoutingWorker';
import type { RouteStep } from '@/routing/types';
import { computed, readonly, ref, watch } from 'vue';

const rallyRoute = ref<RouteStep[]>([]);
let initialized = false;

export function useRallyRoute() {
    if (!initialized) {
        initialized = true;

        const map = useMap();

        const homeSolarsystemId = computed(() => {
            if (!map.value.home_solarsystem_id) return null;
            const homeMapSystem = map.value.map_solarsystems?.find((s) => s.id === map.value.home_solarsystem_id);
            return homeMapSystem?.solarsystem_id ?? null;
        });

        const { routingSettings, convertedEveScoutConnections, getConnections } = useRoutingSetup({
            mapConnections: computed(() => map.value.map_connections ?? []),
            mapSolarsystems: computed(() => map.value.map_solarsystems ?? []),
        });

        watch(
            [
                homeSolarsystemId,
                () => map.value.rally_solarsystem_id,
                () => map.value.map_connections,
                () => map.value.map_solarsystems,
                routingSettings,
                convertedEveScoutConnections,
            ],
            async () => {
                const homeId = homeSolarsystemId.value;
                const rallyId = map.value.rally_solarsystem_id;

                if (!homeId || !rallyId) {
                    rallyRoute.value = [];
                    return;
                }

                await initializeRouting();

                const { dynamicConnections, eveScoutConnections } = getConnections();

                const result = findRoute(routingSettings.value, homeId, rallyId, dynamicConnections, eveScoutConnections, []);

                rallyRoute.value = result.route;
            },
            { immediate: true },
        );
    }

    const rallyRouteSystemIds = computed(() => new Set(rallyRoute.value.map((step) => step.id)));

    function getRallyRouteInfo(fromSolarsystemId: number, toSolarsystemId: number): { onRoute: boolean; reversed: boolean } {
        const route = rallyRoute.value;
        if (route.length < 2) return { onRoute: false, reversed: false };

        for (let i = 0; i < route.length - 1; i++) {
            if (route[i].id === fromSolarsystemId && route[i + 1].id === toSolarsystemId) {
                return { onRoute: true, reversed: false };
            }
            if (route[i].id === toSolarsystemId && route[i + 1].id === fromSolarsystemId) {
                return { onRoute: true, reversed: true };
            }
        }

        return { onRoute: false, reversed: false };
    }

    return {
        rallyRoute: readonly(rallyRoute),
        rallyRouteSystemIds,
        getRallyRouteInfo,
    };
}
