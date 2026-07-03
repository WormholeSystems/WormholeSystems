import EveSearchController from '@/actions/App/Http/Controllers/EveSearchController';
import { TEveSearchResult } from '@/types/models';
import { useHttp } from '@inertiajs/vue3';
import { useDebounceFn } from '@vueuse/core';
import { ref } from 'vue';

export type TEveSearchKind = 'type' | 'group';

type TEveSearchResponse = { data: TEveSearchResult[] };

/**
 * Typeahead against the EVE ship type/group reference data via Inertia's `useHttp`.
 * One instance per input so concurrent rows don't share results; the search and resolve
 * requests use separate handles so an on-mount resolve can't cancel an in-flight search.
 */
export function useEveSearch() {
    const results = ref<TEveSearchResult[]>([]);
    const searchHttp = useHttp<Record<string, never>, TEveSearchResponse>({});
    const resolveHttp = useHttp<Record<string, never>, TEveSearchResponse>({});

    async function run(kind: TEveSearchKind, term: string, filters: { category_id?: number } = {}): Promise<void> {
        const query = term.trim();

        if (query.length < 1) {
            results.value = [];
            return;
        }

        await searchHttp.get(EveSearchController.index.url({ query: { kind, q: query, ...filters } }), {
            onSuccess: (response) => {
                results.value = response?.data ?? [];
            },
        });
    }

    const search = useDebounceFn(run, 200);

    /**
     * Resolve a set of ids to their names, e.g. to label already-saved chips when editing.
     */
    async function resolveLabels(kind: TEveSearchKind, ids: number[]): Promise<Record<number, string>> {
        if (ids.length === 0) {
            return {};
        }

        let labels: Record<number, string> = {};

        await resolveHttp.get(EveSearchController.index.url({ query: { kind, ids } }), {
            onSuccess: (response) => {
                labels = Object.fromEntries((response?.data ?? []).map((result) => [result.id, result.name]));
            },
        });

        return labels;
    }

    return { results, search, resolveLabels };
}
