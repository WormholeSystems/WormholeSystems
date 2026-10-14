import { describe, expect, it } from 'vitest';

import {
	bookmarkClass,
	formatBookmark,
	isReturnBookmark,
	renderBookmark,
	shortSignatureId,
	type BookmarkContext,
	type BookmarkSystem,
} from './bookmark';

const wormhole: BookmarkSystem = {
	alias: 'D2',
	name: 'J122515',
	region: 'E-R00024',
	wormholeClassId: 5,
	security: -0.99,
	occupier: null,
};

const kspace: BookmarkSystem = {
	alias: null,
	name: 'Jita',
	region: 'The Forge',
	wormholeClassId: null,
	security: 0.95,
	occupier: null,
};

const context: BookmarkContext = {
	signatureId: 'ABC-123',
	size: null,
	massStatus: null,
	timeStatus: null,
	wormholeCode: null,
};

describe('bookmarkClass', () => {
	it('uses the wormhole class, and the two-letter form for k-space', () => {
		expect(bookmarkClass(5, -0.99)).toBe('C5');
		expect(bookmarkClass(null, 0.95)).toBe('HS');
		expect(bookmarkClass(null, 0.3)).toBe('LS');
		expect(bookmarkClass(null, -0.5)).toBe('NS');
	});
});

describe('shortSignatureId', () => {
	it('keeps the three characters you actually read in the scanner', () => {
		expect(shortSignatureId('ABC-123')).toBe('ABC');
		expect(shortSignatureId(null)).toBe('');
	});
});

describe('renderBookmark', () => {
	const values = {
		alias: 'D2',
		sig: 'ABC',
		class: 'C5',
		name: 'J122515',
		region: '',
		occupier: '',
		size: '',
		wh: '',
		mass: '',
		life: '',
	};

	it('drops empty tokens and closes the gap they leave', () => {
		expect(renderBookmark('{alias} {sig} {class} {region}', values)).toBe('D2 ABC C5');
	});

	it('leaves an unknown placeholder alone, so a typo is visible', () => {
		expect(renderBookmark('{alias} {nope}', values)).toBe('D2 {nope}');
	});
});

describe('formatBookmark', () => {
	it('uses the wormhole format for wormhole space', () => {
		expect(formatBookmark(wormhole, context)).toBe('D2 ABC C5');
	});

	it('uses the k-space format, which names the system and region', () => {
		expect(formatBookmark(kspace, context)).toBe('HS ABC Jita The Forge');
	});

	it('honours a format the map has set', () => {
		expect(formatBookmark(wormhole, context, { wormhole: '{class} {alias} [{sig}]' })).toBe(
			'C5 D2 [ABC]',
		);
	});

	it('names only the restrictive sizes, large being the one that goes without saying', () => {
		const format = { wormhole: '{alias} {sig} {size}' };
		expect(formatBookmark(wormhole, { ...context, size: 'small' }, format)).toBe('D2 ABC SM');
		expect(formatBookmark(wormhole, { ...context, size: 'medium' }, format)).toBe('D2 ABC MD');
		expect(formatBookmark(wormhole, { ...context, size: 'xl' }, format)).toBe('D2 ABC XM');
		expect(formatBookmark(wormhole, { ...context, size: 'large' }, format)).toBe('D2 ABC');
	});

	it('only shows mass and lifetime once the hole has degraded', () => {
		const fresh = { ...context, massStatus: 'stable' as const, timeStatus: 'stable' as const };
		const dying = { ...context, massStatus: 'critical' as const, timeStatus: 'eol' as const };
		const format = { wormhole: '{alias} {sig} {mass} {life}' };
		expect(formatBookmark(wormhole, fresh, format)).toBe('D2 ABC');
		expect(formatBookmark(wormhole, dying, format)).toBe('D2 ABC crit EOL');
	});

	it.each([
		['critical', 'D2 ABC EOL!'],
		['expired', 'D2 ABC EOL!!'],
	] as const)('marks a %s hole as "%s"', (timeStatus, expected) => {
		const format = { wormhole: '{alias} {sig} {life}' };
		expect(formatBookmark(wormhole, { ...context, timeStatus }, format)).toBe(expected);
	});
});

describe('isReturnBookmark', () => {
	it('treats an ancestor as the way back, since aliases extend their parent', () => {
		expect(isReturnBookmark('A', 'AB', 'HOME')).toBe(true);
	});

	it('leaves a sibling branch pointing forward', () => {
		expect(isReturnBookmark('B', 'AB', 'HOME')).toBe(false);
	});

	it('treats home as the way back from anywhere', () => {
		expect(isReturnBookmark('HOME', 'AB', 'HOME')).toBe(true);
	});

	it('never chooses the return format without the other end to compare to', () => {
		expect(isReturnBookmark('A', null, 'HOME')).toBe(false);
	});

	it('picks the return format for the way home', () => {
		expect(formatBookmark({ ...wormhole, alias: 'A' }, context, null, 'AB')).toBe('*A ABC C5');
	});
});

describe('an unmapped hole', () => {
	it('says only what the signature type promises, and never guesses a class', () => {
		const unknownFarSide: BookmarkSystem = {
			alias: null,
			name: '',
			region: null,
			wormholeClassId: null,
			security: null,
			occupier: null,
		};
		// Nothing is known beyond the scanner id, so the class token drops out entirely
		// rather than resolving to a confident "LS".
		expect(formatBookmark(unknownFarSide, context)).toBe('ABC');
		// With a typed hole, the class it leads to is known even before it is mapped.
		expect(formatBookmark({ ...unknownFarSide, wormholeClassId: 5 }, context)).toBe('ABC C5');
	});
});
