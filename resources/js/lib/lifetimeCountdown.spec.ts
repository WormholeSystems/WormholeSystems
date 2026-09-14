import { lifetimeCountdown } from '@/lib/lifetimeCountdown';
import { describe, expect, it } from 'vitest';

const MARKED = '2026-09-14T12:00:00Z';

function at(offsetMinutes: number): Date {
    return new Date(Date.parse(MARKED) + offsetMinutes * 60 * 1000);
}

describe('lifetimeCountdown', () => {
    it('counts four hours down from the moment a hole was marked end of life', () => {
        expect(lifetimeCountdown('eol', MARKED, at(0))?.label).toBe('4h 0m 0s');
        expect(lifetimeCountdown('eol', MARKED, at(61))?.label).toBe('2h 59m 0s');
    });

    it('counts one hour down for a critical hole', () => {
        expect(lifetimeCountdown('critical', MARKED, at(0))?.label).toBe('1h 0m 0s');
        expect(lifetimeCountdown('critical', MARKED, at(45))?.label).toBe('15m 0s');
    });

    it('drops the hours once under an hour, and the minutes once under a minute', () => {
        expect(lifetimeCountdown('eol', MARKED, at(180.5))?.label).toBe('59m 30s');
        expect(lifetimeCountdown('eol', MARKED, at(239.75))?.label).toBe('15s');
    });

    it('reports a hole that outlived its budget instead of counting backwards', () => {
        const countdown = lifetimeCountdown('eol', MARKED, at(300));

        expect(countdown).toEqual({ remainingMs: 0, expired: true, label: 'any moment now' });
    });

    it('gives a healthy hole no countdown, since nothing promises when it collapses', () => {
        expect(lifetimeCountdown('healthy', MARKED, at(0))).toBeNull();
    });

    it('gives no countdown without a marking timestamp', () => {
        expect(lifetimeCountdown('eol', null, at(0))).toBeNull();
        expect(lifetimeCountdown('eol', 'not a date', at(0))).toBeNull();
    });
});
