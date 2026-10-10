import { uniquePinnedDestinations } from '@/lib/destinations';
import { describe, expect, it } from 'vitest';

function destination(id: number, solarsystem_id: number, is_pinned = true) {
    return { id, solarsystem_id, is_pinned };
}

describe('uniquePinnedDestinations', () => {
    it('keeps pinned entries only', function () {
        expect(uniquePinnedDestinations([destination(1, 30000142), destination(2, 30002187, false)]).map((d) => d.id)).toEqual([1]);
    });

    it('keeps a single entry for a system pinned on both lists', function () {
        expect(uniquePinnedDestinations([destination(1, 30000142), destination(2, 30000142), destination(3, 30002187)]).map((d) => d.id)).toEqual([
            1, 3,
        ]);
    });

    it('applies the limit after deduplication', function () {
        const destinations = [
            destination(1, 30000142),
            destination(2, 30000142),
            destination(3, 30002187),
            destination(4, 30002659),
            destination(5, 30002510),
        ];

        expect(uniquePinnedDestinations(destinations, 3).map((d) => d.id)).toEqual([1, 3, 4]);
    });
});
