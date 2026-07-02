import { useSolarsystemAliases } from '@/composables/useSolarsystemAliases';
import { useStaticData } from '@/composables/useStaticData';
import type { TMapSolarsystemBase } from '@/pages/maps';
import type { TStaticSolarsystem } from '@/types/static-data';
import { computed, MaybeRefOrGetter, toValue } from 'vue';

const MAX_RESULTS = 25;

/**
 * Shared system-search used by the map context menu and the add-connection dialog:
 * filters the static system list by name and splits it into systems not yet on the
 * map and ones that already are. Takes the map's systems as a getter so it isn't
 * tied to a single page shape.
 */
export function useSolarsystemSearch(query: MaybeRefOrGetter<string>, mapSolarsystems: MaybeRefOrGetter<TMapSolarsystemBase[] | undefined>) {
    const { staticData, loadStaticData } = useStaticData();
    const { getAlias } = useSolarsystemAliases(() => toValue(mapSolarsystems) ?? []);

    void loadStaticData();

    const onMapIds = computed(() => new Set((toValue(mapSolarsystems) ?? []).map((map_solarsystem) => map_solarsystem.solarsystem_id)));

    const results = computed(() => {
        const needle = toValue(query).trim().toLowerCase();
        if (!needle) {
            return [] as TStaticSolarsystem[];
        }

        return (staticData.value?.solarsystems ?? []).filter((solarsystem) => solarsystem.name.toLowerCase().includes(needle)).slice(0, MAX_RESULTS);
    });

    const new_solarsystems = computed(() => results.value.filter((solarsystem) => !onMapIds.value.has(solarsystem.id)));
    const existing_solarsystems = computed(() => results.value.filter((solarsystem) => onMapIds.value.has(solarsystem.id)));

    return { new_solarsystems, existing_solarsystems, getAlias };
}
