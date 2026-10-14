import { redirect } from '@sveltejs/kit';
import { pages } from '$lib/docs';

/** `/documentation` is the first page rather than an index nobody reads. */
export function load() {
	const first = pages[0];
	if (first) redirect(307, first.url);
	return {};
}
