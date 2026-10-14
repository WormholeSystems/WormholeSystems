import { describe, expect, it } from 'vitest';

import { LIFETIME_OPTIONS, MASS_OPTIONS, edgeColor, massBadgeColor } from './connection-status';

const mass = (value: string) => MASS_OPTIONS.find((o) => o.value === value)!.color;
const time = (value: string) => LIFETIME_OPTIONS.find((o) => o.value === value)!.color;

describe('edgeColor', () => {
	it('draws a degraded hole in the color of its own badge', () => {
		expect(edgeColor('wormhole', 'reduced', null, false)).toBe(massBadgeColor('reduced'));
		expect(edgeColor('wormhole', 'critical', null, false)).toBe(mass('critical'));
		expect(edgeColor('wormhole', null, 'eol', false)).toBe(time('eol'));
		expect(edgeColor('wormhole', null, 'expired', false)).toBe(time('expired'));
	});

	it('lets the worse state win', () => {
		expect(edgeColor('wormhole', 'reduced', 'critical', false)).toBe(time('critical'));
		expect(edgeColor('wormhole', 'reduced', 'eol', false)).toBe(mass('reduced'));
	});

	it('keeps the route and stargates apart from every status color', () => {
		const route = edgeColor('wormhole', 'critical', 'eol', true);
		const gate = edgeColor('stargate', null, null, false);
		const statuses = [...MASS_OPTIONS, ...LIFETIME_OPTIONS].map((o) => o.color);
		expect(statuses).not.toContain(route);
		expect(statuses).not.toContain(gate);
	});

	it('falls back to the neutral token', () => {
		expect(edgeColor('wormhole', 'stable', 'stable', false)).toBe('var(--color-edge)');
	});
});
