import { guessNextAlias, suggestAlias } from '@/lib/alias';
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
