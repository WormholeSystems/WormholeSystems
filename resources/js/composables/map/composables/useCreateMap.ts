import { applyScale, getConnectionWithSourceAndTarget, getHoveredState, getSelectedState } from '@/composables/map';
import { TLayout } from '@/composables/useLayout';
import { TMap } from '@/pages/maps';
import { TMapConfig } from '@/types/map';
import { MaybeRefOrGetter, toValue, watchEffect } from 'vue';
import { mapState } from '../state';

export function useCreateMap(
    map: MaybeRefOrGetter<TMap>,
    container: MaybeRefOrGetter<HTMLElement>,
    config: MaybeRefOrGetter<TMapConfig>,
    layout?: MaybeRefOrGetter<TLayout>,
) {
    watchEffect(createLayout);
    watchEffect(createMap);
    watchEffect(createConnections);

    function createMap() {
        const mapValue = toValue(map);
        const containerValue = toValue(container);
        if (!mapValue) return;

        const configValue = toValue(config);

        mapState.map = mapValue;
        mapState.map_container = containerValue || null;
        mapState.map_solarsystems = mapValue.map_solarsystems!.map(getSelectedState).map(getHoveredState).map(applyScale);
        mapState.config = configValue;
    }

    function createLayout() {
        const layoutValue = toValue(layout);
        mapState.scale = layoutValue?.scale ?? 1;
    }

    function createConnections() {
        mapState.map_connections = mapState.map!.map_connections!.map(getConnectionWithSourceAndTarget);
    }
}
