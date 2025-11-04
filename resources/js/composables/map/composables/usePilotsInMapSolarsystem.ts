import { useMapCharacters } from '@/composables/useMapCharacters';
import { TMapSolarsystem } from '@/pages/maps';
import { MaybeRefOrGetter } from '@vueuse/core';
import { computed, toValue } from 'vue';

export function usePilotsInMapSolarsystem(map_solarsystem: MaybeRefOrGetter<TMapSolarsystem>) {
    const characters = useMapCharacters();

    return computed(() => {
        const system = toValue(map_solarsystem);
        if (!system || !system.id || !characters) {
            return [];
        }
        return characters.value?.filter((character) => {
            return character.status?.solarsystem_id === system.solarsystem_id;
        });
    });
}
