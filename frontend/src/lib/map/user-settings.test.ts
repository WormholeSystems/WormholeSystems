import { describe, expect, it } from 'vitest';

import type { MapUserSettings } from '$lib/api/types/MapUserSettings';
import {
	AUTOMATION,
	LOCATION_PERMISSION_MISSING,
	automationPatch,
	settingBlocker,
	settingOn,
	settingValue,
	type AutomationKey,
} from './user-settings';

function settings(over: Partial<MapUserSettings>): MapUserSettings {
	return over as MapUserSettings;
}

describe('AUTOMATION', () => {
	// The server's own defaults (src/api/user_settings.rs and the migrations). The popover
	// used to read false for the two that are on, so a loading map showed them off.
	it('defaults the way the server does', () => {
		const defaults = Object.fromEntries(
			Object.values(AUTOMATION).map((s) => [s.key, settingValue(null, s.key)]),
		);
		expect(defaults).toEqual({
			tracking_allowed: false,
			prompt_for_signature: true,
			preselect_signature: false,
			suggest_alias: true,
			copy_bookmark: false,
			follow_character: false,
		});
	});

	it('keys every entry by its own key', () => {
		for (const [key, setting] of Object.entries(AUTOMATION)) expect(setting.key).toBe(key);
	});

	it('says what is missing for every setting another one leans on', () => {
		const leanedOn = new Set(
			Object.values(AUTOMATION)
				.map((s) => s.needs)
				.filter((k): k is AutomationKey => k !== null),
		);
		for (const key of leanedOn) expect(AUTOMATION[key].missing).toBeTruthy();
	});
});

describe('settingBlocker', () => {
	it('blocks location sharing only on the missing permission', () => {
		expect(settingBlocker(null, 'tracking_allowed', false)).toBe(LOCATION_PERMISSION_MISSING);
		expect(settingBlocker(null, 'tracking_allowed', true)).toBeNull();
	});

	it('holds everything else behind location sharing', () => {
		for (const key of Object.keys(AUTOMATION) as AutomationKey[]) {
			if (key === 'tracking_allowed') continue;
			expect(settingBlocker(settings({ tracking_allowed: false }), key)).toBe(
				'Needs location sharing.',
			);
		}
	});

	it('names the first missing link of a chain, from the root down', () => {
		const off = settings({ tracking_allowed: false, prompt_for_signature: false });
		expect(settingBlocker(off, 'preselect_signature')).toBe('Needs location sharing.');

		const sharing = settings({ tracking_allowed: true, prompt_for_signature: false });
		expect(settingBlocker(sharing, 'preselect_signature')).toBe('Needs the signature prompt.');

		const prompting = settings({ tracking_allowed: true, prompt_for_signature: true });
		expect(settingBlocker(prompting, 'preselect_signature')).toBeNull();
	});
});

describe('settingOn', () => {
	it('is off while what it leans on is off, whatever is stored', () => {
		const s = settings({ tracking_allowed: false, suggest_alias: true });
		expect(settingValue(s, 'suggest_alias')).toBe(true);
		expect(settingOn(s, 'suggest_alias')).toBe(false);
	});

	it('takes the permission into account for location sharing itself', () => {
		const s = settings({ tracking_allowed: true });
		expect(settingOn(s, 'tracking_allowed', false)).toBe(false);
		expect(settingOn(s, 'tracking_allowed', true)).toBe(true);
	});
});

describe('automationPatch', () => {
	it('sets the one field', () => {
		expect(automationPatch('copy_bookmark', true)).toEqual({ copy_bookmark: true });
	});
});
