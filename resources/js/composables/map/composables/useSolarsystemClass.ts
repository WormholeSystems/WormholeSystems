import { getSecurityClass } from '@/composables/map';
import { TMapSolarSystem, TSolarsystemClass } from '@/types/models';
import { computed, MaybeRefOrGetter, toValue } from 'vue';

export function useSolarsystemClass(solarsystem: MaybeRefOrGetter<TMapSolarSystem>) {
    return computed<TSolarsystemClass>(() => {
        const system = toValue(solarsystem);
        if (system.class) return system.class as TSolarsystemClass;

        return getSecurityClass(system.solarsystem!.security) as TSolarsystemClass;
    });
}
