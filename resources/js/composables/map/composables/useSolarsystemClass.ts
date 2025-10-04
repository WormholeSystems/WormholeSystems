import { getSecurityClass } from '@/composables/map';
import { TMapSolarSystem } from '@/types/models';
import { computed, MaybeRefOrGetter, toValue } from 'vue';

export function useSolarsystemClass(solarsystem: MaybeRefOrGetter<TMapSolarSystem>) {
    return computed(() => {
        const system = toValue(solarsystem);
        if (system.class) return system.class;

        return getSecurityClass(system.solarsystem!.security);
    });
}
