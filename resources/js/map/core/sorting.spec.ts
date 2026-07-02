import { compareSystems, sortByAlias, sortByClass, sortByName } from '@/map/core/sorting';
import { TMapSolarsystem } from '@/pages/maps';
import { describe, expect, it } from 'vitest';

/**
 * Characterization tests pinning the canonical system ordering (shared by the
 * tree layout's sibling order and the organize action) before it moves into
 * map/core.
 */

function system(overrides: { alias?: string | null; class?: string; region?: string; name?: string }): TMapSolarsystem {
    return {
        alias: overrides.alias ?? null,
        solarsystem: {
            name: overrides.name ?? 'J100001',
            class: overrides.class ?? '3',
            region: { name: overrides.region ?? 'A-R00001' },
        },
    } as unknown as TMapSolarsystem;
}

describe('sortByAlias', () => {
    it('sorts systems without an alias before aliased ones', () => {
        expect(sortByAlias(system({ alias: 'HOME' }), system({}))).toBe(1);
        expect(sortByAlias(system({}), system({ alias: 'HOME' }))).toBe(-1);
    });

    it('compares two aliases lexicographically and treats two missing aliases as equal', () => {
        expect(sortByAlias(system({ alias: 'A' }), system({ alias: 'B' }))).toBeLessThan(0);
        expect(sortByAlias(system({}), system({}))).toBe(0);
    });
});

describe('sortByClass', () => {
    it('orders known space before wormhole space', () => {
        expect(sortByClass(system({ class: 'h' }), system({ class: '1' }))).toBeLessThan(0);
        expect(sortByClass(system({ class: 'n' }), system({ class: 'l' }))).toBeGreaterThan(0);
    });

    it('orders wormhole classes numerically', () => {
        expect(sortByClass(system({ class: '1' }), system({ class: '6' }))).toBeLessThan(0);
    });
});

describe('compareSystems', () => {
    it('applies alias, then class, then region, then name', () => {
        const unsorted = [
            system({ alias: 'Z', name: 'J1' }),
            system({ class: '5', name: 'J2' }),
            system({ class: '5', name: 'J3', region: 'B-R00002' }),
            system({ class: 'h', name: 'Jita' }),
            system({ class: '5', name: 'J2', region: 'B-R00002' }),
        ];

        const sorted = [...unsorted].sort(compareSystems);
        expect(sorted.map((s) => s.solarsystem.name)).toEqual(['Jita', 'J2', 'J2', 'J3', 'J1']);
        // Region breaks the tie between the two J2/J3 C5s before name does.
        expect(sorted[1].solarsystem.region!.name).toBe('A-R00001');
    });

    it('is stable for identical systems', () => {
        expect(compareSystems(system({}), system({}))).toBe(0);
    });
});

describe('sortByName', () => {
    it('compares system names lexicographically', () => {
        expect(sortByName(system({ name: 'J100001' }), system({ name: 'J200002' }))).toBeLessThan(0);
    });
});
