import { describe, expect, it } from 'vitest';
import type { SignatureTypeInfo } from '$lib/api/types/SignatureTypeInfo';
import { matchesTypeQuery } from './type-search';

const type = (over: Partial<SignatureTypeInfo>): SignatureTypeInfo =>
	({
		id: 1,
		signature: null,
		name: '',
		signature_category_id: 1,
		target_class: null,
		extra: null,
		spawn_areas: [],
		...over,
	}) as SignatureTypeInfo;

const h296 = type({ id: 223, signature: 'H296', name: 'H296 - C5', target_class: 5 });
const v753 = type({ id: 224, signature: 'V753', name: 'V753 - C6', target_class: 6 });
const c140 = type({ id: 225, signature: 'C140', name: 'C140 - Lowsec', target_class: 8 });
const e004 = type({
	id: 226,
	signature: 'E004',
	name: 'E004 - C1',
	target_class: 1,
	extra: 'frigate',
});
const site = type({ id: 300, name: 'Vast Frontier Reservoir' });

describe('matchesTypeQuery', () => {
	it('matches the wormhole code', () => {
		expect(matchesTypeQuery(h296, 'h296')).toBe(true);
		expect(matchesTypeQuery(h296, 'H29')).toBe(true);
		expect(matchesTypeQuery(v753, 'h296')).toBe(false);
	});

	it('finds every hole to a class by that class', () => {
		// The point of the whole change: `C6` means "holes that go to a C6", and a plain
		// select could not answer it because every label starts with the code instead.
		expect([h296, v753, c140].filter((t) => matchesTypeQuery(t, 'c6'))).toEqual([v753]);
	});

	it('matches anywhere in the name, not only the start', () => {
		expect(matchesTypeQuery(c140, 'lowsec')).toBe(true);
		expect(matchesTypeQuery(site, 'frontier')).toBe(true);
	});

	it('takes several terms, matching different things', () => {
		expect(matchesTypeQuery(e004, 'c1 frigate')).toBe(true);
		expect(matchesTypeQuery(e004, 'frigate c1')).toBe(true);
		expect(matchesTypeQuery(e004, 'c1 cruiser')).toBe(false);
	});

	it('ignores case and stray whitespace', () => {
		expect(matchesTypeQuery(v753, '  V753  ')).toBe(true);
		expect(matchesTypeQuery(v753, 'c6')).toBe(matchesTypeQuery(v753, 'C6'));
	});

	it('matches everything on an empty query', () => {
		expect([h296, site].every((t) => matchesTypeQuery(t, ''))).toBe(true);
		expect(matchesTypeQuery(site, '   ')).toBe(true);
	});
});
