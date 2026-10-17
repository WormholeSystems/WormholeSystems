import { describe, expect, it } from 'vitest';

import { parseScan as parseWith, typeById } from './signatures';
import type { SignatureCatalog } from '$lib/api/types/SignatureCatalog';
import type { SignatureTypeInfo } from '$lib/api/types/SignatureTypeInfo';
import translations from '$lib/signatures/translations.json';

/** The parser with the shipped names, which the app loads on the first paste. */
function parseScan(text: string, catalog: SignatureCatalog) {
	return parseWith(text, catalog, translations);
}

function type(
	partial: Partial<SignatureTypeInfo> &
		Pick<SignatureTypeInfo, 'id' | 'name' | 'signature_category_id'>,
): SignatureTypeInfo {
	return {
		signature: null,
		target_class: null,
		extra: null,
		spawn_areas: [],
		total_mass: null,
		max_jump_mass: null,
		lifetime_hours: null,
		signature_strength: null,
		...partial,
	};
}

// Category ids follow the seeded catalog: 1 wormhole, 2 data, 3 relic, 5 gas.
const catalog: SignatureCatalog = {
	categories: [
		{ id: 1, name: 'Wormhole', code: 'WH' },
		{ id: 2, name: 'Data Site', code: 'DATA' },
		{ id: 3, name: 'Relic Site', code: 'RELIC' },
		{ id: 5, name: 'Gas Site', code: 'GAS' },
	],
	types: [
		type({ id: 10, name: 'Unstable Wormhole', signature_category_id: 1, signature: 'K162' }),
		type({ id: 20, name: 'Unsecured Frontier Receiver', signature_category_id: 2 }),
		type({ id: 30, name: 'Forgotten Perimeter Coronation Site', signature_category_id: 3 }),
	],
};

describe('parseScan', () => {
	it('keeps only the signatures from a real combat probe paste', () => {
		// Eight rows off the scanner: two signatures, four ships and two drones. The ships
		// carry scan ids in the same shape, so nothing but the group tells them apart.
		const paste = [
			'DTB-020\tCosmic Signature\t\t\t5.3%\t7.55 AU',
			'GQF-170\tDrone\t\t\t13.7%\t8.70 AU',
			'LWJ-023\tShip\t\t\t2.7%\t10.00 AU',
			'NSR-149\tShip\t\t\t17.6%\t1.53 AU',
			'RXX-515\tCosmic Signature\t\t\t19.3%\t1.88 AU',
			'SOH-202\tShip\tCarrier\tNidhoggur\t77.4%\t10.51 AU',
			'VPP-145\tDrone\t\t\t13.7%\t9.03 AU',
			'YEN-272\tShip\t\t\t3.4%\t2.31 AU',
		].join('\n');
		expect(parseScan(paste, catalog).map((r) => r.signature_id)).toEqual(['DTB-020', 'RXX-515']);
	});

	it('drops structures and deployables too', () => {
		const paste = [
			'GHI-789\tStructure\tUpwell Palatine Keepstar\tKeepstar\t100%\t1.2 AU',
			'MNO-345\tDeployable\tMobile Depot\tMobile Depot\t100%\t30 km',
		].join('\n');
		expect(parseScan(paste, catalog)).toEqual([]);
	});

	it('keeps a row whose group it does not recognise, so localised clients still work', () => {
		// `translations` has no group names, so a German client's "Kosmische Signatur" is
		// not matched. Keeping it is what those players get today.
		const rows = parseScan('PQR-678\tKosmische Signatur\t\t\t0%\t6.51 AU', catalog);
		expect(rows.map((r) => r.signature_id)).toEqual(['PQR-678']);
	});

	it('parses a multi-line scanner paste', () => {
		const paste = [
			'ABC-123\tCosmic Signature\tWormhole\tUnstable Wormhole\t100.0%\t2.5 AU',
			'DEF-456\tCosmic Signature\tData Site\tUnsecured Frontier Receiver\t97.2%\t11.1 AU',
			'GHI-789\tCosmic Signature\tRelic Site\tForgotten Perimeter Coronation Site\t54.8%\t3.2 AU',
		].join('\n');

		expect(parseScan(paste, catalog)).toEqual([
			// A wormhole's type is never taken from the paste: the scanner only ever says
			// "Unstable Wormhole", so the raw name is kept and the user picks the type.
			{
				signature_id: 'ABC-123',
				group: 'wormhole',
				signature_type_id: undefined,
				name: 'Unstable Wormhole',
			},
			{ signature_id: 'DEF-456', group: 'data', signature_type_id: 20, name: undefined },
			{ signature_id: 'GHI-789', group: 'relic', signature_type_id: 30, name: undefined },
		]);
	});

	it('keeps partially scanned rows: no category, or a category with no type yet', () => {
		const paste = [
			'JKL-012\tCosmic Signature\t\t\t2.9%\t8.7 AU',
			'MNO-345\tCosmic Signature\tGas Site\t\t12.0%\t1.4 AU',
		].join('\n');

		expect(parseScan(paste, catalog)).toEqual([
			{ signature_id: 'JKL-012', group: undefined, signature_type_id: undefined, name: undefined },
			{ signature_id: 'MNO-345', group: 'gas', signature_type_id: undefined, name: undefined },
		]);
	});

	it('keeps an unmatched type name as the raw name', () => {
		const rows = parseScan(
			'PQR-678\tCosmic Signature\tData Site\tSome New Site\t80%\t1 AU',
			catalog,
		);
		expect(rows).toEqual([
			{
				signature_id: 'PQR-678',
				group: 'data',
				signature_type_id: undefined,
				name: 'Some New Site',
			},
		]);
	});

	it('matches a category by its " - " segments when the full cell is not a category name', () => {
		const rows = parseScan('STU-901\tCosmic Signature\tGuristas - Data Site\t\t9%\t3 AU', catalog);
		expect(rows).toEqual([
			{ signature_id: 'STU-901', group: 'data', signature_type_id: undefined, name: undefined },
		]);
	});

	it('skips malformed lines and keeps the rest', () => {
		const paste = [
			'',
			'not a scanner line at all',
			'AB-12\tCosmic Signature\tWormhole\tUnstable Wormhole\t100%\t1 AU',
			'ABC-123\tCosmic Signature',
			'VWX-234\tCosmic Signature\tRelic Site\tForgotten Perimeter Coronation Site\t54.8%\t3.2 AU',
		].join('\n');

		expect(parseScan(paste, catalog)).toEqual([
			{ signature_id: 'VWX-234', group: 'relic', signature_type_id: 30, name: undefined },
		]);
	});

	it('files faction warfare anomalies under their category with the pasted type name', () => {
		const fwCatalog: SignatureCatalog = {
			...catalog,
			categories: [
				...catalog.categories,
				{ id: 6, name: 'Ore Site', code: 'ore' },
				{ id: 8, name: 'Factional Warfare Site', code: 'faction-warfare' },
			],
		};
		const paste = [
			'BBL-893\tCosmic Anomaly\tFactional Warfare Site - Combat Site\tAmarr Scout BSC-1\t100,0%\t6,84 AU',
			'CBA-620\tCosmic Anomaly\tOre Site\tGlacial Mass Belt\t100,0%\t7,02 AU',
			'DXA-556\tCosmic Signature\t\t\t0,0%\t6,51 AU',
			'MSA-264\tCosmic Anomaly\tFactional Warfare Site - Combat Site\tAmarr Moderate NVY-3\t100,0%\t60,60 AU',
		].join('\n');

		expect(parseScan(paste, fwCatalog)).toEqual([
			{
				signature_id: 'BBL-893',
				group: 'faction_warfare',
				signature_type_id: undefined,
				name: 'Amarr Scout BSC-1',
			},
			{
				signature_id: 'CBA-620',
				group: 'ore',
				signature_type_id: undefined,
				name: 'Glacial Mass Belt',
			},
			{ signature_id: 'DXA-556', group: undefined, signature_type_id: undefined, name: undefined },
			{
				signature_id: 'MSA-264',
				group: 'faction_warfare',
				signature_type_id: undefined,
				name: 'Amarr Moderate NVY-3',
			},
		]);
	});

	it('parses nothing from empty input', () => {
		expect(parseScan('', catalog)).toEqual([]);
	});
	describe('localized clients', () => {
		// The real catalog's ids and names for the sites these pastes mention.
		const localized: SignatureCatalog = {
			categories: [
				{ id: 1, name: 'Wormhole', code: 'wormhole' },
				{ id: 2, name: 'Data Site', code: 'data' },
				{ id: 3, name: 'Relic Site', code: 'relic' },
				{ id: 4, name: 'Combat Site', code: 'combat' },
				{ id: 5, name: 'Gas Site', code: 'gas' },
				{ id: 6, name: 'Ore Site', code: 'ore' },
				{ id: 8, name: 'Factional Warfare Site', code: 'faction-warfare' },
			],
			types: [
				type({ id: 124, name: 'Unsecured Perimeter Amplifier', signature_category_id: 2 }),
				type({ id: 65, name: 'Ruined Sansha Temple Site', signature_category_id: 3 }),
				type({ id: 17, name: 'Core Garrison', signature_category_id: 4 }),
				type({ id: 35, name: 'Unstable Wormhole', signature_category_id: 4 }),
				type({ id: 136, name: 'Barren Perimeter Reservoir', signature_category_id: 5 }),
			],
		};

		it('parses a German paste with decimal commas', () => {
			const paste = [
				'ABC-123\tKosmische Signatur\tDatengebiet\tUngesicherter Verstärker im Außenbereich\t100,0%\t4,2 AE',
				'DEF-456\tKosmische Signatur\tWurmloch\tInstabiles Wurmloch\t100,0%\t12,34 AE',
				'GHI-789\tKosmische Signatur\t\t\t0,0%\t6,51 AE',
			].join('\n');

			expect(parseScan(paste, localized)).toEqual([
				{ signature_id: 'ABC-123', group: 'data', signature_type_id: 124, name: undefined },
				{
					signature_id: 'DEF-456',
					group: 'wormhole',
					signature_type_id: undefined,
					name: 'Instabiles Wurmloch',
				},
				{
					signature_id: 'GHI-789',
					group: undefined,
					signature_type_id: undefined,
					name: undefined,
				},
			]);
		});

		it('parses Russian and Chinese pastes', () => {
			const paste = [
				'ABC-123\tСкрытый сигнал\tАрхеологический район\tРазрушенный храмовый комплекс Sansha\t100,0%\t3,10 а.е.',
				'DEF-456\t空间信号\t战斗地点\t核心兵站\t100.0%\t5.03 AU',
			].join('\n');

			expect(parseScan(paste, localized)).toEqual([
				{ signature_id: 'ABC-123', group: 'relic', signature_type_id: 65, name: undefined },
				{ signature_id: 'DEF-456', group: 'combat', signature_type_id: 17, name: undefined },
			]);
		});

		it('parses a clipboard mixing several languages', () => {
			const paste = [
				'AAA-111\tCosmic Signature\tGas Site\tBarren Perimeter Reservoir\t100.0%\t5.03 AU',
				'BBB-222\tKosmische Signatur\tGasgebiet\tKarges Gasvorkommen\t100,0%\t4,2 AE',
				'CCC-333\t코즈믹 시그니처\t가스 사이트\t황량한 변방 가스 매장지\t100.0%\t1.20 AU',
				'DDD-444\t宇宙のシグネチャ\tガスサイト\t荒地の防御ライン貯水池\t100.0%\t2.40 AU',
				'EEE-555\tSignature cosmique\tSite de collecte de gaz\tRéservoir de périmètre aride\t100,0%\t7,00 UA',
				'FFF-666\tSeñal cósmica\tZona de gas\tReserva perimetral baldía\t100,0%\t8,00 UA',
			].join('\n');

			const rows = parseScan(paste, localized);
			expect(rows).toHaveLength(6);
			for (const row of rows) {
				expect(row).toMatchObject({ group: 'gas', signature_type_id: 136, name: undefined });
			}
		});

		it('recognizes localized faction warfare sites', () => {
			const line =
				'BUH-704\tКосмическая аномалия\tРайон прохождения межгосударственных войн - Боевой район\tМинматарский объект\t100,0%\t13,83 а.е.';

			expect(parseScan(line, localized)).toEqual([
				{
					signature_id: 'BUH-704',
					group: 'faction_warfare',
					signature_type_id: undefined,
					name: 'Минматарский объект',
				},
			]);
		});

		it('treats non-breaking spaces like regular spaces', () => {
			const line =
				'ABC-123\tКосмическая аномалия\tБоевой\u00a0район\tНеустойчивая\u00a0червоточина\t100,0%\t1,00 а.е.';

			expect(parseScan(line, localized)).toEqual([
				{ signature_id: 'ABC-123', group: 'combat', signature_type_id: 35, name: undefined },
			]);
		});

		it('keeps the raw name when a translation is shared by several sites', () => {
			const line =
				'ABC-123\tKosmische Anomalie\tMineraliengebiet\tGewöhnliche Erzvorkommen im Grenzgebiet\t100,0%\t1,00 AE';

			expect(parseScan(line, localized)).toEqual([
				{
					signature_id: 'ABC-123',
					group: 'ore',
					signature_type_id: undefined,
					name: 'Gewöhnliche Erzvorkommen im Grenzgebiet',
				},
			]);
		});
	});
});

describe('typeById', () => {
	it('finds a type by id, and nothing for an unknown or missing one', () => {
		expect(typeById(catalog, 20)?.name).toBe('Unsecured Frontier Receiver');
		expect(typeById(catalog, 99)).toBeNull();
		expect(typeById(catalog, null)).toBeNull();
	});

	it('answers from each catalog it is handed', () => {
		const other: SignatureCatalog = {
			...catalog,
			types: [type({ id: 20, name: 'Renamed', signature_category_id: 2 })],
		};
		typeById(catalog, 20);
		expect(typeById(other, 20)?.name).toBe('Renamed');
	});
});
