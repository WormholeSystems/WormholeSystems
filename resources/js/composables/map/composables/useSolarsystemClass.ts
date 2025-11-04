import { getSecurityClass } from '@/composables/map';
import { TMapSolarsystem } from '@/pages/maps';
import { TSolarsystemClass } from '@/types/models';
import { computed, MaybeRefOrGetter, toValue } from 'vue';

export function useSolarsystemClass(solarsystem: MaybeRefOrGetter<TMapSolarsystem>) {
    return computed<TSolarsystemClass>(() => {
        const system = toValue(solarsystem);
        if (system.solarsystem.class) return system.solarsystem.class as TSolarsystemClass;

        return getSecurityClass(system.solarsystem!.security) as TSolarsystemClass;
    });
}
