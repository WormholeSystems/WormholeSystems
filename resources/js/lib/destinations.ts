import type { TMapRouteSolarsystem } from '@/types/models';

/**
 * The pinned watchlist entries, one per solarsystem: a system pinned on both the shared and
 * the personal list must only take one quick-pick slot.
 */
export function uniquePinnedDestinations<T extends Pick<TMapRouteSolarsystem, 'solarsystem_id' | 'is_pinned'>>(
    destinations: T[],
    limit?: number,
): T[] {
    const seen = new Set<number>();
    const pinned: T[] = [];

    for (const destination of destinations) {
        if (!destination.is_pinned || seen.has(destination.solarsystem_id)) {
            continue;
        }

        seen.add(destination.solarsystem_id);
        pinned.push(destination);

        if (pinned.length === limit) {
            break;
        }
    }

    return pinned;
}
