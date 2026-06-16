import type { TMapSolarsystemBase } from '@/pages/maps';
import { computed, type MaybeRefOrGetter, toValue } from 'vue';

export function useSolarsystemAliases(mapSolarsystems: MaybeRefOrGetter<TMapSolarsystemBase[] | undefined>) {
    const aliases = computed(() => {
        const lookup = new Map<number, string>();

        for (const mapSolarsystem of toValue(mapSolarsystems) ?? []) {
            if (mapSolarsystem.alias) {
                lookup.set(mapSolarsystem.solarsystem_id, mapSolarsystem.alias);
            }
        }

        return lookup;
    });

    function getAlias(solarsystemId: number): string | null {
        return aliases.value.get(solarsystemId) ?? null;
    }

    return { aliases, getAlias };
}
