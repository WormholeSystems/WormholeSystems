import { convertEveScoutConnections, convertMapConnectionsToWorkerEdges } from '@/composables/routing/utils';
import { useEveScoutConnections } from '@/composables/useEveScoutConnections';
import { useMapUserSettings } from '@/composables/useMapUserSettings';
import type { TMapConnection, TMapSolarsystem } from '@/pages/maps';
import type { RoutingConnection, RoutingSettings } from '@/routing/types';
import type { ComputedRef, MaybeRefOrGetter } from 'vue';
import { computed, toValue } from 'vue';

type UseRoutingSetupParams = {
    mapConnections: MaybeRefOrGetter<TMapConnection[]>;
    mapSolarsystems: MaybeRefOrGetter<TMapSolarsystem[]>;
    includeEveScout?: MaybeRefOrGetter<boolean>;
};

export function useRoutingSetup(params: UseRoutingSetupParams) {
    const mapUserSettings = useMapUserSettings();
    const eveScoutConnections = useEveScoutConnections();

    const useEveScout = computed(() => {
        if (params.includeEveScout !== undefined) {
            return toValue(params.includeEveScout) && mapUserSettings.value.route_use_evescout;
        }
        return mapUserSettings.value.route_use_evescout;
    });

    const routingSettings: ComputedRef<RoutingSettings> = computed(() => ({
        routePreference: mapUserSettings.value.route_preference,
        securityPenalty: mapUserSettings.value.security_penalty,
        lifetimeStatus: mapUserSettings.value.route_allow_lifetime_status,
        massStatus: mapUserSettings.value.route_allow_mass_status,
        useEveScout: useEveScout.value,
    }));

    const convertedConnections = computed(() => convertMapConnectionsToWorkerEdges(toValue(params.mapConnections), toValue(params.mapSolarsystems)));

    const convertedEveScoutConnections = computed(() => convertEveScoutConnections(eveScoutConnections.value, useEveScout.value));

    function getConnections(): { dynamicConnections: RoutingConnection[]; eveScoutConnections: RoutingConnection[] } {
        return {
            dynamicConnections: convertedConnections.value.map((edge) => ({ ...edge })),
            eveScoutConnections: convertedEveScoutConnections.value.map((edge) => ({ ...edge })),
        };
    }

    return {
        routingSettings,
        convertedConnections,
        convertedEveScoutConnections,
        getConnections,
    };
}
