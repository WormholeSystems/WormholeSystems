import { describe, expect, it } from 'vitest';

import { routeChoicePatch } from './options';

describe('routeChoicePatch', () => {
	it('patches the field the picker belongs to', () => {
		expect(routeChoicePatch('route_preference', 'safer')).toEqual({ route_preference: 'safer' });
		expect(routeChoicePatch('route_allow_time_status', 'eol')).toEqual({
			route_allow_time_status: 'eol',
		});
		expect(routeChoicePatch('route_allow_mass_status', 'stable')).toEqual({
			route_allow_mass_status: 'stable',
		});
	});

	it("refuses a value from another field's list", () => {
		// `eol` is a lifetime, not a mass status, and must not reach the server as one.
		expect(routeChoicePatch('route_allow_mass_status', 'eol')).toBeNull();
		expect(routeChoicePatch('route_preference', '')).toBeNull();
	});
});
