import { guessNextAlias, isIgnoredAlias, suggestAlias } from '@/lib/alias';
import { describe, expect, it } from 'vitest';

describe('guessNextAlias (numeric, default)', () => {
    it('numbers top-level systems sequentially', () => {
        expect(guessNextAlias(null, [])).toBe('1');
        expect(guessNextAlias(null, ['1'])).toBe('2');
        expect(guessNextAlias(null, ['1', '2'])).toBe('3');
    });

    it('extends the parent prefix for children', () => {
        expect(guessNextAlias('1', [])).toBe('11');
        expect(guessNextAlias('1', ['11', '12'])).toBe('13');
        expect(guessNextAlias('12', ['12', '121'])).toBe('122');
    });

    it('excludes grandchildren when counting direct children', () => {
        // "121" and "122" are children of "12", not of "1" — only "12" itself
        // (index 2) counts as a direct child of "1", so the next is "13", not
        // something inflated by the grandchildren.
        expect(guessNextAlias('1', ['1', '12', '121', '122'])).toBe('13');
    });
});

describe('guessNextAlias (alphabetical)', () => {
    const scheme = 'alphabetical' as const;

    it('letters an unaliased home\'s direct holes, mirroring numeric\'s 1, 2, 3', () => {
        expect(guessNextAlias(null, [], { scheme })).toBe('A');
        expect(guessNextAlias(null, ['A'], { scheme })).toBe('B');
        expect(guessNextAlias(null, ['A', 'B'], { scheme })).toBe('C');
    });

    it('extends the parent prefix with a letter', () => {
        expect(guessNextAlias('A', [], { scheme })).toBe('AA');
        expect(guessNextAlias('A', ['AA'], { scheme })).toBe('AB');
    });

    it('skips the reserved H, L, N, P letters', () => {
        expect(guessNextAlias('A', ['AA', 'AB', 'AC', 'AD', 'AE', 'AF', 'AG'], { scheme })).toBe('AI');
        expect(guessNextAlias('A', ['AA', 'AB', 'AC', 'AD', 'AE', 'AF', 'AG', 'AI', 'AJ', 'AK'], { scheme })).toBe('AM');
        expect(guessNextAlias('A', ['AA', 'AB', 'AC', 'AD', 'AE', 'AF', 'AG', 'AI', 'AJ', 'AK', 'AM'], { scheme })).toBe('AO');
    });

    it('indexes k-space exits per reserved letter, off the parent', () => {
        expect(guessNextAlias('A', [], { scheme, targetKind: 'h' })).toBe('AH1');
        expect(guessNextAlias('A', ['AH1'], { scheme, targetKind: 'h' })).toBe('AH2');
        expect(guessNextAlias('AC', [], { scheme, targetKind: 'l' })).toBe('ACL1');
        expect(guessNextAlias('B', ['BN1'], { scheme, targetKind: 'n' })).toBe('BN2');
        expect(guessNextAlias('A', [], { scheme, targetKind: 'p' })).toBe('AP1');
    });

    it('indexes a k-space exit off an unaliased home the same way, empty prefix and all', () => {
        expect(guessNextAlias(null, [], { scheme, targetKind: 'h' })).toBe('H1');
        expect(guessNextAlias(null, ['H1'], { scheme, targetKind: 'h' })).toBe('H2');
    });

    it('keeps the wormhole letter sequence and each k-space counter independent for mixed children', () => {
        const aliases = ['AB', 'AH1', 'AL1'];

        expect(guessNextAlias('A', aliases, { scheme })).toBe('AC');
        expect(guessNextAlias('A', aliases, { scheme, targetKind: 'h' })).toBe('AH2');
        expect(guessNextAlias('A', aliases, { scheme, targetKind: 'l' })).toBe('AL2');
    });

    it('lets a wormhole branch off a k-space node', () => {
        expect(guessNextAlias('AH1', [], { scheme })).toBe('AH1A');
    });
});

describe('guessNextAlias (ignoredAlias)', () => {
    it('resets the prefix when the parent is the ignored alias, alphabetical scheme', () => {
        expect(guessNextAlias('HOME', [], { scheme: 'alphabetical', ignoredAlias: 'HOME' })).toBe('A');
        expect(guessNextAlias('HOME', ['A'], { scheme: 'alphabetical', ignoredAlias: 'HOME' })).toBe('B');
    });

    it('resets the prefix when the parent is the ignored alias, numeric scheme', () => {
        expect(guessNextAlias('HOME', [], { ignoredAlias: 'HOME' })).toBe('1');
        expect(guessNextAlias('HOME', ['1'], { ignoredAlias: 'HOME' })).toBe('2');
    });

    it('matches the ignored alias case-insensitively', () => {
        expect(guessNextAlias('home', [], { scheme: 'alphabetical', ignoredAlias: 'HOME' })).toBe('A');
        expect(guessNextAlias('Home', [], { scheme: 'alphabetical', ignoredAlias: 'HOME' })).toBe('A');
    });

    it('leaves a non-ignored parent unaffected', () => {
        expect(guessNextAlias('AB', [], { scheme: 'alphabetical', ignoredAlias: 'HOME' })).toBe('ABA');
        expect(guessNextAlias('AB', [], { ignoredAlias: 'HOME' })).toBe('AB1');
    });

    it('is a no-op when ignoredAlias is empty', () => {
        expect(guessNextAlias('HOME', [], { scheme: 'alphabetical', ignoredAlias: '' })).toBe('HOMEA');
        expect(guessNextAlias('HOME', [], { ignoredAlias: '' })).toBe('HOME1');
    });
});

describe('guessNextAlias (case handling)', () => {
    it('upper-cases the suggestion even for a lowercase parent alias', () => {
        expect(guessNextAlias('a', [], { scheme: 'alphabetical' })).toBe('AA');
        expect(guessNextAlias('foo', [])).toBe('FOO1');
    });

    it('counts lowercase existing aliases as taken wormhole letters', () => {
        expect(guessNextAlias('A', ['aa', 'Ab'], { scheme: 'alphabetical' })).toBe('AC');
    });

    it('counts lowercase existing aliases as taken k-space indexes', () => {
        expect(guessNextAlias('a', ['ah1'], { scheme: 'alphabetical', targetKind: 'h' })).toBe('AH2');
    });

    it('counts lowercase existing aliases as taken numeric children', () => {
        expect(guessNextAlias('foo', ['foo1', 'FOO2'])).toBe('FOO3');
    });
});

describe('isIgnoredAlias', () => {
    it('matches the exact, trimmed, case-insensitive alias', () => {
        expect(isIgnoredAlias('HOME', 'HOME')).toBe(true);
        expect(isIgnoredAlias('home', 'HOME')).toBe(true);
        expect(isIgnoredAlias(' Home ', 'home')).toBe(true);
    });

    it('does not match a different alias', () => {
        expect(isIgnoredAlias('AB', 'HOME')).toBe(false);
        expect(isIgnoredAlias('HOMEA', 'HOME')).toBe(false);
    });

    it('never matches when the alias is empty', () => {
        expect(isIgnoredAlias('', 'HOME')).toBe(false);
        expect(isIgnoredAlias(null, 'HOME')).toBe(false);
        expect(isIgnoredAlias(undefined, 'HOME')).toBe(false);
    });

    it('never matches when the ignored alias is empty (feature disabled)', () => {
        expect(isIgnoredAlias('HOME', '')).toBe(false);
        expect(isIgnoredAlias('HOME', null)).toBe(false);
        expect(isIgnoredAlias('HOME', undefined)).toBe(false);
    });
});

describe('suggestAlias', () => {
    it('threads the scheme and target kind through to guessNextAlias', () => {
        const alias = suggestAlias({
            parentAlias: 'A',
            targetIsWormhole: false,
            originIsWormhole: true,
            aliases: ['A'],
            scheme: 'alphabetical',
            targetKind: 'h',
        });

        expect(alias).toBe('AH1');
    });

    it('still returns null when neither side of the jump is part of a chain', () => {
        const alias = suggestAlias({
            parentAlias: null,
            targetIsWormhole: false,
            originIsWormhole: false,
            aliases: [],
            scheme: 'alphabetical',
        });

        expect(alias).toBeNull();
    });
});
