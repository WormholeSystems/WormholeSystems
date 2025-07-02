import { TMap } from '@/types/models';
import { computed, ComputedRef, inject, provide, useTemplateRef } from 'vue';

const map_symbol = Symbol('useMap');

type TMapContext = ComputedRef<{
    map: TMap;
    container: HTMLDivElement | null;
}>;

export function useMap(map?: TMap): TMapContext {
    if (map) {
        return createMap(map);
    }

    return inject<TMapContext>(map_symbol)!;
}

export function createMap(map: TMap) {
    const container = useTemplateRef<HTMLDivElement>('map-container');

    const data: TMapContext = computed(() => ({
        map,
        container: container.value,
    }));

    provide(map_symbol, data);

    return data;
}
