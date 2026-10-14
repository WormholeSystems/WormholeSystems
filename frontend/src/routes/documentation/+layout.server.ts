import { categories } from '$lib/docs';

/** The sidebar is the same on every page, so it is resolved once for the section. */
export function load() {
	return { categories };
}
