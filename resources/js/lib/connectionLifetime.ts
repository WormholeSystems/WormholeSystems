import { isDrifterClass, isWormholeClass } from '@/const/solarsystemClasses';
import { formatRemaining, lifetimeCountdown, type LifetimeCountdown } from '@/lib/lifetimeCountdown';
import type { TLifetimeStatus, TStringedSolarsystemClass } from '@/types/models';

const HOUR_MS = 60 * 60 * 1000;

type Endpoints = {
    from: TStringedSolarsystemClass | null | undefined;
    to: TStringedSolarsystemClass | null | undefined;
};

/**
 * How long a hole of this shape lives in total: drifter holes leaving wormhole
 * space run 16 hours, C6 statics leaving it 48, everything else the usual 24.
 *
 * Mirrored on the backend in App\Console\Commands\CheckConnectionAgeCommand,
 * which flips the lifetime status off the same thresholds. It reads the split as
 * one end having a wormhole system record and the other not, so anything outside
 * J-space counts as the far end, abyssal and Pochven included.
 */
export function connectionLifetimeMs({ from, to }: Endpoints): number {
    const leavesWormholeSpace = isWormholeClass(from) !== isWormholeClass(to);

    if (leavesWormholeSpace && (isDrifterClass(from) || isDrifterClass(to))) {
        return 16 * HOUR_MS;
    }

    if (leavesWormholeSpace && (from === '6' || to === '6')) {
        return 48 * HOUR_MS;
    }

    return 24 * HOUR_MS;
}

export type TimeRemaining = LifetimeCountdown & {
    /** The marking drove the number, so it is firmer than an age estimate. */
    fromMarking: boolean;
};

type TimeRemainingInput = {
    /** When the hole reached the map, the closest thing we have to when it spawned. */
    startedAt: string | null | undefined;
    lifetimeStatus: TLifetimeStatus | null | undefined;
    lifetimeStatusUpdatedAt: string | null | undefined;
};

/**
 * How much longer a hole should live, shown whatever its status rather than only
 * once someone flags it.
 *
 * This follows the same rule the background command uses to move a connection's
 * status along: a hole already marked end of life or critical is timed from that
 * marking, because someone read it off the hole itself, and anything else is
 * timed from its age against the lifetime its shape implies.
 */
export function connectionTimeRemaining(input: TimeRemainingInput, endpoints: Endpoints, now: Date): TimeRemaining | null {
    const marked = lifetimeCountdown(input.lifetimeStatus, input.lifetimeStatusUpdatedAt, now);
    if (marked) {
        return { ...marked, fromMarking: true };
    }

    // The command only ages connections that touch wormhole space; a k-space to
    // k-space hole has no shape to infer a lifetime from.
    if (!isWormholeClass(endpoints.from) && !isWormholeClass(endpoints.to)) {
        return null;
    }

    if (!input.startedAt) {
        return null;
    }

    const started = new Date(input.startedAt);
    if (Number.isNaN(started.getTime())) {
        return null;
    }

    const remainingMs = started.getTime() + connectionLifetimeMs(endpoints) - now.getTime();

    if (remainingMs <= 0) {
        return { remainingMs: 0, expired: true, label: 'any moment now', fromMarking: false };
    }

    return { remainingMs, expired: false, label: formatRemaining(remainingMs), fromMarking: false };
}
