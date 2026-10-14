import { SvelteMap } from 'svelte/reactivity';

import { api } from '$lib/api/client';
import type { SystemSearchResult } from '$lib/api/types/SystemSearchResult';

/**
 * A grow-only, request-deduplicating cache over a batch resolver. Factory-shaped so a
 * test can construct a fresh one with a stub fetcher; the app shares the singleton below.
 */
/** The server truncates a resolve request at this many ids; bigger asks are split. */
const CHUNK = 200;

export function createSystemResolver(fetchRows: (ids: number[]) => Promise<SystemSearchResult[]>) {
	// Per-key reactivity: a batch landing wakes the reads of the ids it brought (and of ids
	// still missing), not every row that already had its answer.
	const resolved = new SvelteMap<number, SystemSearchResult>();
	// One promise per id currently on the wire, so concurrent asks share the fetch.
	const pending = new Map<number, Promise<void>>();
	// Ids a successful fetch did not answer: the server does not know them, and asking
	// again would loop forever, since a reactive read retriggers on every cache change.
	const unknown = new Set<number>();

	async function fetchMissing(ids: number[]): Promise<void> {
		const waits: Promise<void>[] = [];
		const missing: number[] = [];
		for (const id of new Set(ids)) {
			if (resolved.has(id) || unknown.has(id)) continue;
			const wait = pending.get(id);
			if (wait) waits.push(wait);
			else missing.push(id);
		}
		for (let at = 0; at < missing.length; at += CHUNK) {
			const chunk = missing.slice(at, at + CHUNK);
			const batch = fetchRows(chunk)
				.then((rows) => {
					const answered = new Set(rows.map((row) => row.id));
					for (const id of chunk) {
						if (!answered.has(id) && !resolved.has(id)) unknown.add(id);
					}
					// A row that is already known is not written again: a write wakes its
					// readers, which would only ask again.
					for (const row of rows) {
						if (!resolved.has(row.id)) resolved.set(row.id, row);
					}
				})
				.catch(() => {})
				.finally(() => {
					for (const id of chunk) pending.delete(id);
				});
			for (const id of chunk) pending.set(id, batch);
			waits.push(batch);
		}
		await Promise.all(waits);
	}

	// Asks are collected per microtask and flushed as one batch: every miss a single
	// render pass reads coalesces into a single request instead of one per system.
	let queued = new Set<number>();
	let flushScheduled = false;
	function enqueue(ids: number[]) {
		for (const id of ids) {
			if (!resolved.has(id) && !pending.has(id) && !unknown.has(id)) queued.add(id);
		}
		if (queued.size === 0 || flushScheduled) return;
		flushScheduled = true;
		queueMicrotask(() => {
			flushScheduled = false;
			const batch = [...queued];
			queued = new Set();
			void fetchMissing(batch);
		});
	}

	return {
		/** What has arrived (or been seeded) for `id`; undefined until then. */
		get(id: number): SystemSearchResult | undefined {
			return resolved.get(id);
		},
		/** Fire-and-forget: fetch whatever of `ids` is neither known nor already on the wire. */
		ensure(ids: number[]) {
			enqueue(ids);
		},
		/** As `ensure`, awaitable for one id. Never throws; undefined when the fetch fails. */
		async resolve(id: number): Promise<SystemSearchResult | undefined> {
			await fetchMissing([id]);
			return resolved.get(id);
		},
		/** Hand over rows another payload already carried, so nobody fetches them again. */
		seed(rows: Iterable<SystemSearchResult>) {
			for (const row of rows) {
				if (!resolved.has(row.id)) resolved.set(row.id, row);
			}
		},
	};
}

/**
 * The one cache behind `api.resolveSystems`. Every surface that needs display data for a
 * solar system asks here, so a system goes over the wire at most once per session no
 * matter how many panels name it. The map page seeds it with its placed systems, which
 * therefore never hit the network at all.
 */
export const systemResolver = createSystemResolver((ids) => api.resolveSystems(ids));
