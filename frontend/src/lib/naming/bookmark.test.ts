import { describe, expect, it } from 'vitest';

import type { MapConnection } from '$lib/api/types/MapConnection';
import type { MapSystemView } from '$lib/api/types/MapSystemView';
import type { Signature } from '$lib/api/types/Signature';
import type { MappedSystem } from '$lib/map/system';
import {
	bookmarkClass,
	bookmarkSystemOf,
	connectionBookmarks,
	formatBookmark,
	isReturnBookmark,
	renderBookmark,
	shortSignatureId,
	signatureBookmark,
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
	isHome: false,
};

const kspace: BookmarkSystem = {
	alias: null,
	name: 'Jita',
	region: 'The Forge',
	wormholeClassId: null,
	security: 0.95,
	occupier: null,
	isHome: false,
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
		current: '',
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

	it('names the system the bookmark is made in with {current}', () => {
		const formats = { wormhole: '{alias} {sig} {current}', return: '*{alias} {sig} {current}' };
		expect(formatBookmark({ ...wormhole, alias: 'AB' }, context, formats, 'A')).toBe('AB ABC A');
		expect(formatBookmark({ ...wormhole, alias: 'A' }, context, formats, 'AB')).toBe('*A ABC AB');
		expect(formatBookmark(wormhole, context, formats)).toBe('D2 ABC');
	});

	it('treats the home system as the way back even without an alias', () => {
		const home = { ...kspace, name: 'Turnur', region: 'Metropolis', security: 0.4, isHome: true };
		expect(formatBookmark(home, context, null, '1')).toBe('* ABC LS');
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
			isHome: false,
		};
		// Nothing is known beyond the scanner id, so the class token drops out entirely
		// rather than resolving to a confident "LS".
		expect(formatBookmark(unknownFarSide, context)).toBe('ABC');
		// With a typed hole, the class it leads to is known even before it is mapped.
		expect(formatBookmark({ ...unknownFarSide, wormholeClassId: 5 }, context)).toBe('ABC C5');
	});
});

function placed(over: Partial<MappedSystem>): MappedSystem {
	return {
		kind: 'system',
		alias: null,
		region: null,
		occupying_group: null,
		is_home: false,
		...over,
	} as MappedSystem;
}

function sig(over: Partial<Signature>): Signature {
	return {
		signature_id: 'AAA-000',
		signature_type_id: null,
		connection_id: null,
		size: null,
		mass_status: null,
		time_status: null,
		...over,
	} as Signature;
}

// Jita (alias A) to a C5 (alias A1), the one hole scanned from both ends.
const home = placed({
	id: 1,
	solar_system_id: 30000142,
	name: 'Jita',
	alias: 'A',
	region: 'The Forge',
	security_status: 0.95,
	wormhole_class_id: null,
});
const c5 = placed({
	id: 2,
	solar_system_id: 31000005,
	name: 'J122515',
	alias: 'A1',
	region: 'E-R00024',
	security_status: -0.99,
	wormhole_class_id: 5,
});
const hole = {
	id: 7,
	from_system: home.id,
	to_system: c5.id,
	size: null,
	mass_status: null,
	time_status: null,
} as MapConnection;
const scannedAtHome = sig({
	signature_id: 'HOM-111',
	solar_system_id: home.solar_system_id,
	connection_id: hole.id,
	signature_type_id: 1,
});
const scannedInC5 = sig({
	signature_id: 'FAR-222',
	solar_system_id: c5.solar_system_id,
	connection_id: hole.id,
	signature_type_id: 2,
});
const codes = (id: number | null) => (id === 1 ? 'H296' : id === 2 ? 'K162' : null);

describe('bookmarkSystemOf', () => {
	it('reads a placed system the way a bookmark names it', () => {
		expect(bookmarkSystemOf(c5)).toEqual({
			alias: 'A1',
			name: 'J122515',
			region: 'E-R00024',
			wormholeClassId: 5,
			security: -0.99,
			occupier: null,
			isHome: false,
		});
	});
});

describe('connectionBookmarks', () => {
	const formats = {
		wormhole: '{alias} {sig} {wh}',
		kspace: '{alias} {sig} {wh}',
		return: '*{alias} {sig} {wh}',
	};

	it("names each end with the signature read on the other, the scanner's side", () => {
		const [toHome, toC5] = connectionBookmarks(
			hole,
			[home, c5],
			[scannedAtHome, scannedInC5],
			codes,
			formats,
		);
		// Standing in the C5, the way back to Jita carries the C5's own signature.
		expect(toHome).toBe('*A FAR K162');
		// Standing in Jita, the way into the C5 carries the one scanned in Jita.
		expect(toC5).toBe('A1 HOM H296');
	});

	it('names the way back to a home system that has no alias as a return', () => {
		const unaliasedHome = { ...home, alias: null, is_home: true };
		const [toHome] = connectionBookmarks(
			hole,
			[unaliasedHome, c5],
			[scannedAtHome, scannedInC5],
			codes,
			formats,
		);
		expect(toHome).toBe('* FAR K162');
	});

	it('leaves the signature out while the near side has not been scanned', () => {
		const names = connectionBookmarks(hole, [home, c5], [scannedInC5], codes, formats);
		expect(names[1]).toBe('A1');
	});

	it('names no ghost, and nothing at all while an end is missing', () => {
		const ghost = { kind: 'ghost', id: 3, alias: 'A2' } as MapSystemView;
		// Only Jita has a system to name, and nothing can have been scanned in a ghost.
		expect(
			connectionBookmarks({ ...hole, to_system: 3 }, [home, ghost], [], codes, formats),
		).toEqual(['*A']);
		expect(connectionBookmarks(hole, [home], [], codes, formats)).toEqual([]);
	});
});

describe('signatureBookmark', () => {
	it('falls back to the class the type promises while the far side is unmapped', () => {
		const text = signatureBookmark(
			sig({ signature_id: 'XYZ-987' }),
			{ signature: 'H296', target_class: 5 },
			null,
			'B',
			'A',
			{ wormhole: '{alias} {sig} {class} {name} {wh}' },
		);
		expect(text).toBe('B XYZ C5 H296');
	});

	it('names the far side once it is on the map', () => {
		const text = signatureBookmark(sig({ signature_id: 'XYZ-987' }), null, c5, 'A1', 'A', {
			wormhole: '{alias} {sig} {class} {name}',
		});
		expect(text).toBe('A1 XYZ C5 J122515');
	});
});
