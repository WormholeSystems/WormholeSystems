import { connectionLifetimeMs, connectionTimeRemaining } from '@/lib/connectionLifetime';
import type { TLifetimeStatus, TStringedSolarsystemClass } from '@/types/models';
import { describe, expect, it } from 'vitest';

const HOUR = 60 * 60 * 1000;
const STARTED = '2026-09-14T00:00:00Z';

function at(offsetHours: number): Date {
    return new Date(Date.parse(STARTED) + offsetHours * HOUR);
}

function remaining(
    hours: number,
    endpoints: { from: TStringedSolarsystemClass; to: TStringedSolarsystemClass },
    marking: { lifetimeStatus?: TLifetimeStatus; lifetimeStatusUpdatedAt?: string } = {},
) {
    return connectionTimeRemaining(
        {
            startedAt: STARTED,
            lifetimeStatus: marking.lifetimeStatus ?? 'healthy',
            lifetimeStatusUpdatedAt: marking.lifetimeStatusUpdatedAt ?? null,
        },
        endpoints,
        at(hours),
    );
}

describe('connectionLifetimeMs', () => {
    it.each<[TStringedSolarsystemClass, TStringedSolarsystemClass, number]>([
        ['5', 'h', 24],
        ['6', 'h', 48],
        ['6', 'n', 48],
        ['6', '5', 24],
        ['6', '6', 24],
        ['14', 'n', 16],
        ['14', '5', 24],
        ['5', '3', 24],
    ])('gives a C%s to %s hole %ih', (from, to, hours) => {
        expect(connectionLifetimeMs({ from, to })).toBe(hours * HOUR);
    });

    it('does not care which end is which', () => {
        expect(connectionLifetimeMs({ from: 'l', to: '6' })).toBe(connectionLifetimeMs({ from: '6', to: 'l' }));
    });
});

/**
 * The thresholds CheckConnectionAgeCommand flips status on. If the command's
 * table changes and this one does not, the countdown would disagree with the
 * status sitting right above it in the popover.
 */
describe('agreement with the background command', () => {
    it.each<[string, TStringedSolarsystemClass, TStringedSolarsystemClass, number, number]>([
        ['drifter leaving wormhole space', '14', 'n', 12, 15],
        ['C6 leaving wormhole space', '6', 'h', 44, 47],
        ['everything else', '5', '3', 20, 23],
    ])('has %s at four hours left on the eol threshold and one on the critical one', (_label, from, to, eolHour, criticalHour) => {
        expect(remaining(eolHour, { from, to })?.remainingMs).toBe(4 * HOUR);
        expect(remaining(criticalHour, { from, to })?.remainingMs).toBe(HOUR);
    });
});

describe('connectionTimeRemaining', () => {
    it('counts an unmarked hole down from its shape, not from a status', () => {
        expect(remaining(6, { from: '5', to: '3' })?.label).toBe('18h 0m');
        expect(remaining(6, { from: '5', to: '3' })?.fromMarking).toBe(false);
    });

    it('times a marked hole from the marking, the way the command does', () => {
        const marked = remaining(2.5, { from: '5', to: '3' }, { lifetimeStatus: 'eol', lifetimeStatusUpdatedAt: at(2).toISOString() });

        expect(marked?.label).toBe('3h 30m');
        expect(marked?.fromMarking).toBe(true);
    });

    it('gives a critical hole its last hour from the marking', () => {
        const marked = remaining(2.5, { from: '5', to: '3' }, { lifetimeStatus: 'critical', lifetimeStatusUpdatedAt: at(2).toISOString() });

        expect(marked?.label).toBe('30m');
    });

    it('reports a hole that outlived its estimate instead of counting backwards', () => {
        expect(remaining(30, { from: '5', to: '3' })).toMatchObject({ remainingMs: 0, expired: true, label: 'any moment now' });
    });

    it.each<[string, TStringedSolarsystemClass, TStringedSolarsystemClass]>([
        ['between two known space systems', 'h', 'l'],
        ['whose systems are missing from the static data', 'unknown', 'unknown'],
        ['with only one end resolved', 'unknown', '5'],
    ])('still counts a hole %s down from the ordinary day', (_label, from, to) => {
        expect(remaining(6, { from, to })?.label).toBe('18h 0m');
    });

    it('says nothing when the start date is unusable and nothing is marked', () => {
        expect(
            connectionTimeRemaining(
                { startedAt: 'not a date', lifetimeStatus: 'healthy', lifetimeStatusUpdatedAt: null },
                { from: '5', to: '3' },
                at(1),
            ),
        ).toBeNull();
    });
});
