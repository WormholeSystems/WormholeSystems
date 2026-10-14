import { describe, expect, it } from 'vitest';

import type { MapConnection } from '$lib/api/types/MapConnection';

import { describeRemaining, formatRemaining, lifetimeDeadline } from './lifetime';

const T0 = Date.parse('2026-09-03T10:00:00Z');
const HOUR = 3_600_000;

function hole(over: Partial<MapConnection> = {}) {
	return {
		kind: 'wormhole' as const,
		time_status: null,
		time_status_updated_at: null,
		created_at: new Date(T0).toISOString(),
		lifetime_hours: 24,
		...over,
	};
}

describe('lifetimeDeadline', () => {
	it('runs four hours from an EOL mark and one from a critical one', () => {
		const marked = new Date(T0 + 2 * HOUR).toISOString();
		expect(lifetimeDeadline(hole({ time_status: 'eol', time_status_updated_at: marked }))).toEqual({
			at: T0 + 6 * HOUR,
			estimated: false,
		});
		expect(
			lifetimeDeadline(hole({ time_status: 'critical', time_status_updated_at: marked })),
		).toEqual({ at: T0 + 3 * HOUR, estimated: false });
	});

	it('has an expired hole due the moment it was marked', () => {
		const marked = new Date(T0 + 2 * HOUR).toISOString();
		expect(
			lifetimeDeadline(hole({ time_status: 'expired', time_status_updated_at: marked })),
		).toEqual({ at: T0 + 2 * HOUR, estimated: false });
	});

	it('estimates from the class lifetime before any mark', () => {
		expect(lifetimeDeadline(hole())).toEqual({ at: T0 + 24 * HOUR, estimated: true });
		expect(lifetimeDeadline(hole({ lifetime_hours: 48 }))).toEqual({
			at: T0 + 48 * HOUR,
			estimated: true,
		});
	});

	it('has nothing to say about a stargate, which is permanent', () => {
		expect(lifetimeDeadline(hole({ kind: 'stargate' }))).toBeNull();
	});

	it('still counts a hole of unreadable shape down from the ordinary day', () => {
		expect(lifetimeDeadline(hole({ lifetime_hours: null }))).toEqual({
			at: T0 + 24 * HOUR,
			estimated: true,
		});
	});

	// The background ageing marks EOL at four hours left and critical at one, so the
	// estimate has to read exactly that when the mark lands.
	it.each([
		['an ordinary hole', 24, 20, 23],
		['a C6 hole into known space', 48, 44, 47],
		['a drifter hole into known space', 16, 12, 15],
	])('agrees with the ageing on %s', (_label, lifetime, eolAt, criticalAt) => {
		const deadline = lifetimeDeadline(hole({ lifetime_hours: lifetime }))!;
		expect(deadline.at - (T0 + eolAt * HOUR)).toBe(4 * HOUR);
		expect(deadline.at - (T0 + criticalAt * HOUR)).toBe(HOUR);
	});
});

describe('formatRemaining', () => {
	it('rounds up to the minute and never goes negative', () => {
		expect(formatRemaining(T0 + 2 * HOUR + 13 * 60_000 + 1, T0)).toBe('2h 14m');
		expect(formatRemaining(T0 + 45 * 60_000, T0)).toBe('45m');
		expect(formatRemaining(T0 - HOUR, T0)).toBe('0m');
	});
});

describe('describeRemaining', () => {
	it('reads like formatRemaining until the deadline passes', () => {
		expect(describeRemaining(T0 + 18 * HOUR, T0)).toBe('18h 0m');
		expect(describeRemaining(T0, T0)).toBe('any moment now');
		expect(describeRemaining(T0 - HOUR, T0)).toBe('any moment now');
	});
});
