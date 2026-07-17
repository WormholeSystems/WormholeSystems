import { useSolarsystemAliases } from '@/composables/useSolarsystemAliases';
import { useStaticData } from '@/composables/useStaticData';
import { MAX_SEARCH_RESULTS, takeRanked } from '@/lib/searchRank';
import type { TMapSolarsystemBase } from '@/pages/maps';
import type { TStaticSolarsystem } from '@/types/static-data';
import { computed, MaybeRefOrGetter, toValue } from 'vue';

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

        return takeRanked(
            staticData.value?.solarsystems ?? [],
            needle,
            MAX_SEARCH_RESULTS,
            (solarsystem) => [solarsystem.name],
            (solarsystem) => solarsystem.name,
        );
    });

    const partitioned = computed(() => {
        const new_solarsystems: TStaticSolarsystem[] = [];
        const existing_solarsystems: TStaticSolarsystem[] = [];

        for (const solarsystem of results.value) {
            (onMapIds.value.has(solarsystem.id) ? existing_solarsystems : new_solarsystems).push(solarsystem);
        }

        return { new_solarsystems, existing_solarsystems };
    });

    const new_solarsystems = computed(() => partitioned.value.new_solarsystems);
    const existing_solarsystems = computed(() => partitioned.value.existing_solarsystems);

    return { new_solarsystems, existing_solarsystems, getAlias };
}
