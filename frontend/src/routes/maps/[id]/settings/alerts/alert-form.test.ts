import { describe, expect, it } from 'vitest';

import { jumpRangeLy } from '$lib/alerts/vocabulary';
import { isValidAlert, parseIds, toSaveAlert, type AlertDraft } from './alert-form';

const draft = (over: Partial<AlertDraft> = {}): AlertDraft => ({
	name: 'Hunters nearby',
	kind: 'killmail',
	delivery: 'webhook',
	webhookId: 4,
	mention: 'none',
	roleRef: null,
	target: null,
	origin: null,
	maxJumps: 5,
	shipType: 'dreadnought',
	jdcLevel: 5,
	filters: [],
	filterMatch: 'any',
	...over,
});

describe('jumpRangeLy', () => {
	// The server's formula: base × (1 + 0.2 × JDC). Drift here mislabels every alert.
	it.each([
		['dreadnought', 0, '3.5'],
		['dreadnought', 5, '7.0'],
		['supercarrier', 5, '6.0'],
		['titan', 4, '5.4'],
		['jump_freighter', 5, '10.0'],
		['rorqual', 0, '5.0'],
		['black_ops', 5, '8.0'],
		['carrier', 3, '5.6'],
		['force_auxiliary', 5, '7.0'],
	] as const)('%s at JDC %i reaches %s ly', (ship, jdc, ly) => {
		expect(jumpRangeLy(ship, jdc)).toBe(ly);
	});
});

describe('parseIds', () => {
	it('reads a messy comma list and drops what is not a positive id', () => {
		expect(parseIds(' 99003581, 1354830081 ,, abc, -4, 0 ')).toEqual([99003581, 1354830081]);
		expect(parseIds('')).toEqual([]);
	});
});

describe('isValidAlert', () => {
	it('needs a name, and a webhook when delivering to one', () => {
		expect(isValidAlert(draft())).toBe(true);
		expect(isValidAlert(draft({ name: '  ' }))).toBe(false);
		expect(isValidAlert(draft({ webhookId: null }))).toBe(false);
	});

	it('needs a target for everything but killmail, and a role when pinging one', () => {
		expect(isValidAlert(draft({ kind: 'proximity' }))).toBe(false);
		expect(isValidAlert(draft({ kind: 'proximity', target: 30000142 }))).toBe(true);
		expect(isValidAlert(draft({ mention: 'role' }))).toBe(false);
		expect(isValidAlert(draft({ mention: 'role', roleRef: 7 }))).toBe(true);
	});
});

describe('toSaveAlert', () => {
	it('strips what the kind does not use', () => {
		const killmail = toSaveAlert(
			draft({
				target: 30000142,
				filters: [{ subject: 'alliance', side: 'either', mode: 'include', ids: [] }],
			}),
		);
		expect(killmail.target_solar_system_id).toBeUndefined();
		expect(killmail.ship_type).toBeUndefined();
		// Empty rules never reach the server.
		expect(killmail.filters).toEqual([]);

		const jump = toSaveAlert(draft({ kind: 'jump_range', target: 30000142 }));
		expect(jump.ship_type).toBe('dreadnought');
		expect(jump.jdc_level).toBe(5);
		expect(jump.filters).toEqual([]);
	});

	it('keeps the starting point for proximity alerts only', () => {
		const near = toSaveAlert(draft({ kind: 'proximity', target: 30000142, origin: 30000144 }));
		expect(near.origin_solar_system_id).toBe(30000144);
		expect(
			toSaveAlert(draft({ kind: 'proximity', target: 30000142 })).origin_solar_system_id,
		).toBeUndefined();
		expect(
			toSaveAlert(draft({ kind: 'jump_range', target: 30000142, origin: 30000144 }))
				.origin_solar_system_id,
		).toBeUndefined();
	});

	it('carries the role only when the mention asks for one', () => {
		expect(toSaveAlert(draft({ roleRef: 7 })).map_webhook_role_id).toBeUndefined();
		expect(toSaveAlert(draft({ mention: 'role', roleRef: 7 })).map_webhook_role_id).toBe(7);
	});
});
