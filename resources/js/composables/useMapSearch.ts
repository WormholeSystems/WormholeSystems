import MapSearchController from '@/actions/App/Http/Controllers/MapSearchController';
import { TNoteSearchResult, TOccupierSearchResult, TThreatSearchResult } from '@/types/models';
import { useHttp } from '@inertiajs/vue3';
import { useDebounceFn } from '@vueuse/core';
import { type MaybeRefOrGetter, ref, toValue } from 'vue';

type TMapSearchResponse = {
    threats: TThreatSearchResult[];
    notes: TNoteSearchResult[];
    occupiers: TOccupierSearchResult[];
};

/** The palette's server-side search; the backend omits notes for viewers. */
export function useMapSearch(mapSlug: MaybeRefOrGetter<string>) {
    const threat_results = ref<TThreatSearchResult[]>([]);
    const note_results = ref<TNoteSearchResult[]>([]);
    const occupier_results = ref<TOccupierSearchResult[]>([]);
    const http = useHttp<Record<string, never>, TMapSearchResponse>({});

    // useHttp does not abort in-flight requests; only the latest run may write results.
    let latest_run = 0;

    async function run(term: string): Promise<void> {
        const query = term.trim();
        const current_run = ++latest_run;

        if (query.length < 2) {
            http.cancel();
            threat_results.value = [];
            note_results.value = [];
            occupier_results.value = [];
            return;
        }

        try {
            const response = await http.get(MapSearchController.index.url({ map: toValue(mapSlug) }, { query: { q: query } }));

            if (current_run !== latest_run) {
                return;
            }

            threat_results.value = response?.threats ?? [];
            note_results.value = response?.notes ?? [];
            occupier_results.value = response?.occupiers ?? [];
        } catch {
            // Cancelled or failed requests keep the previous results.
        }
    }

    const search = useDebounceFn(run, 200);

    return { threat_results, note_results, occupier_results, search };
}
