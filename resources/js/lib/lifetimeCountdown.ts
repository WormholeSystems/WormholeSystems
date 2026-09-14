import type { TLifetimeStatus } from '@/types/models';

/**
 * What the two marked lifetime states promise: EOL is the in-game "reaching the
 * end of its natural lifetime" message, critical the last hour before collapse.
 * A healthy hole has no known collapse time, so it gets no countdown.
 */
const LIFETIME_BUDGET_MS: Partial<Record<TLifetimeStatus, number>> = {
    eol: 4 * 60 * 60 * 1000,
    critical: 60 * 60 * 1000,
};

export type LifetimeCountdown = {
    /** Milliseconds left on the budget, floored at zero. */
    remainingMs: number;
    /** The budget ran out, so the hole is living on borrowed time. */
    expired: boolean;
    label: string;
};

/**
 * The time left before a hole marked end of life or critical is due to collapse,
 * counted from the moment it was marked. Null when nothing can be counted: a
 * healthy hole, or one whose marking carries no timestamp.
 */
export function lifetimeCountdown(
    status: TLifetimeStatus | null | undefined,
    markedAt: string | Date | null | undefined,
    now: Date,
): LifetimeCountdown | null {
    if (!status || !markedAt) {
        return null;
    }

    const budget = LIFETIME_BUDGET_MS[status];
    if (budget === undefined) {
        return null;
    }

    const marked = markedAt instanceof Date ? markedAt : new Date(markedAt);
    if (Number.isNaN(marked.getTime())) {
        return null;
    }

    const remainingMs = marked.getTime() + budget - now.getTime();

    if (remainingMs <= 0) {
        return { remainingMs: 0, expired: true, label: 'any moment now' };
    }

    return { remainingMs, expired: false, label: formatRemaining(remainingMs) };
}

/** A duration down to the minute; seconds would only make the row flicker. */
export function formatRemaining(remainingMs: number): string {
    const totalMinutes = Math.floor(remainingMs / 60_000);
    const hours = Math.floor(totalMinutes / 60);
    const minutes = totalMinutes % 60;

    if (hours > 0) {
        return `${hours}h ${minutes}m`;
    }

    if (minutes > 0) {
        return `${minutes}m`;
    }

    return 'under a minute';
}
