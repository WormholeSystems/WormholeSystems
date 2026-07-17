/**
 * The shared cap for search-driven pickers. Results render through virtualized
 * lists, so the cap no longer protects the DOM — it only bounds the ranking
 * sort for very short queries.
 */
export const MAX_SEARCH_RESULTS = 500;

/** Exact (0) < prefix (1) < substring (2), null when absent. Needle must be trimmed and lowercased. */
export function rankMatch(value: string | null | undefined, needle: string): number | null {
    if (!value) {
        return null;
    }

    const haystack = value.toLowerCase();

    if (!haystack.includes(needle)) {
        return null;
    }

    if (haystack === needle) {
        return 0;
    }

    return haystack.startsWith(needle) ? 1 : 2;
}

/** Top matches by best-matching field, ties broken by name. */
export function takeRanked<T>(
    items: readonly T[],
    needle: string,
    limit: number,
    fields: (item: T) => (string | null | undefined)[],
    name: (item: T) => string,
): T[] {
    const ranked: { item: T; rank: number }[] = [];

    for (const item of items) {
        let best: number | null = null;

        for (const field of fields(item)) {
            const rank = rankMatch(field, needle);

            if (rank !== null && (best === null || rank < best)) {
                best = rank;
            }
        }

        if (best !== null) {
            ranked.push({ item, rank: best });
        }
    }

    return ranked
        .sort((a, b) => a.rank - b.rank || name(a.item).localeCompare(name(b.item)))
        .slice(0, limit)
        .map((entry) => entry.item);
}
