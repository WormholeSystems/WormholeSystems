// Matching a signature type as somebody types at the picker.

import type { SignatureTypeInfo } from '$lib/api/types/SignatureTypeInfo';
import { destClassMeta } from '$lib/map/classes';

/**
 * Whether a type matches what has been typed: its wormhole code, its name, where it leads
 * and whatever the catalog notes about it.
 *
 * Every term has to match something, but they need not match the same thing, so `c6 frig`
 * finds the frigate hole to class 6 without anyone having to remember which order the
 * picker lists them in. An empty query matches everything.
 *
 * The destination is matched because that is how people talk about these holes: a scout
 * reads `C6` off the map and wants the holes that go there, not the one whose name
 * happens to begin with those letters. Prefix matching on the label -- which is what a
 * plain select gives you -- cannot answer that: the labels all start with the code.
 */
export function matchesTypeQuery(type: SignatureTypeInfo, query: string): boolean {
	const terms = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
	if (terms.length === 0) return true;

	const haystack = [
		type.signature ?? '',
		type.name,
		type.extra ?? '',
		type.target_class === null ? '' : destClassMeta(type.target_class).short,
	]
		.join(' ')
		.toLowerCase();

	return terms.every((term) => haystack.includes(term));
}
