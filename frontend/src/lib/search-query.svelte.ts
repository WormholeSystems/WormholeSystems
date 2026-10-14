import {
	createQuery,
	keepPreviousData,
	type CreateQueryOptions,
	type QueryKey,
} from '@tanstack/svelte-query';

import { debounced } from '$lib/debounced.svelte';

/**
 * The search-as-you-type shape every picker shares: a debounced term feeding a keyed
 * query, gated on a minimum length, with the previous list kept painted while the next
 * one fetches. Keyed by term, so a slow reply can never land on a newer search.
 * Component init only.
 */
export function searchQuery<T, K extends QueryKey>(opts: {
	term: () => string;
	query: (settled: string) => CreateQueryOptions<T[], Error, T[], K>;
	enabled?: () => boolean;
	minChars?: number;
	/** 0 skips the timer entirely, for surfaces that want every keystroke. */
	debounceMs?: number;
}) {
	const minChars = opts.minChars ?? 2;
	const delay = opts.debounceMs ?? 150;
	const settled =
		delay > 0
			? debounced(() => opts.term().trim(), delay)
			: {
					get current() {
						return opts.term().trim();
					},
				};
	const query = createQuery(() => ({
		...opts.query(settled.current),
		enabled: (opts.enabled?.() ?? true) && settled.current.length >= minChars,
		placeholderData: keepPreviousData,
	}));
	return {
		get settled() {
			return settled.current;
		},
		/** Whether the live input is long enough to search on. */
		get searching() {
			return opts.term().trim().length >= minChars;
		},
		get results(): T[] {
			if (!this.searching || !(opts.enabled?.() ?? true)) return [];
			return query.data ?? [];
		},
	};
}
