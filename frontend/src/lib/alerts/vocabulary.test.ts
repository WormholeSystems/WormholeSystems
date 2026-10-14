import { describe, expect, it } from 'vitest';

import type { MapAlert } from '$lib/api/types/MapAlert';
import { alertSummary, jumpRangeLy } from './vocabulary';

function alert(over: Partial<MapAlert>): MapAlert {
	return { kind: 'proximity', max_jumps: 3, filters: [], filter_match: 'any', ...over } as MapAlert;
}

describe('alertSummary', () => {
	it('says how close a system has to come, counting jumps in the singular too', () => {
		expect(alertSummary(alert({ target_system_name: 'Jita', max_jumps: 1 }))).toBe(
			'Jita within 1 jump',
		);
		expect(alertSummary(alert({ target_system_name: 'Jita' }))).toBe('Jita within 3 jumps');
	});

	it('names the origin when the distance is measured from one', () => {
		expect(alertSummary(alert({ target_system_name: 'Jita', origin_system_name: 'J122515' }))).toBe(
			'Jita within 3 jumps of J122515, through the chain',
		);
	});

	it('falls back to "a system" while the target is unnamed', () => {
		expect(alertSummary(alert({}))).toBe('a system within 3 jumps');
	});

	it('counts the filters on a kill alert, or says anything goes without one', () => {
		expect(alertSummary(alert({ kind: 'killmail' }))).toBe('Anything that dies within 3 jumps');
		const filters = [{}, {}] as MapAlert['filters'];
		expect(alertSummary(alert({ kind: 'killmail', filters, filter_match: 'all' }))).toBe(
			'2 filters (all), within 3 jumps',
		);
	});

	it('describes a jump range alert by hull and skill, not by jumps', () => {
		expect(
			alertSummary(
				alert({
					kind: 'jump_range',
					ship_type: 'titan',
					jdc_level: 5,
					target_system_name: 'Amarr',
				}),
			),
		).toBe('An exit within Titan range (JDC 5) of Amarr');
		expect(alertSummary(alert({ kind: 'jump_range' }))).toBe(
			'An exit within a capital range (JDC 0) of a system',
		);
	});
});

describe('jumpRangeLy', () => {
	it('adds a fifth of the base range per JDC level', () => {
		expect(jumpRangeLy('dreadnought', 0)).toBe('3.5');
		expect(jumpRangeLy('dreadnought', 5)).toBe('7.0');
		expect(jumpRangeLy('jump_freighter', 4)).toBe('9.0');
	});
});
