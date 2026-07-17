import { groupSignatureOptions, signatureCanBeConnection, signatureCanLeadToClass } from '@/lib/signatureCompatibility';
import type { TSignature, TSignatureCategory, TSignatureType, TStringedSolarsystemClass } from '@/types/models';
import { describe, expect, it } from 'vitest';

function signature(overrides: {
    id?: number;
    categoryCode?: string;
    targetClass?: TStringedSolarsystemClass | 'unknown' | null;
    hasType?: boolean;
    connected?: boolean;
}): TSignature {
    const { id = 1, categoryCode, targetClass = null, hasType = targetClass !== null, connected = false } = overrides;

    return {
        id,
        signature_id: 'ABC-123',
        map_connection_id: connected ? id : null,
        signature_category: categoryCode ? ({ id: 1, name: categoryCode, code: categoryCode } as TSignatureCategory) : null,
        signature_type: hasType ? ({ id: 1, name: 'Test', signature: 'X702', target_class: targetClass } as TSignatureType) : null,
    } as TSignature;
}

describe('signatureCanBeConnection', () => {
    it('accepts wormhole-category signatures', () => {
        expect(signatureCanBeConnection(signature({ categoryCode: 'wormhole' }))).toBe(true);
    });

    it('accepts uncategorised signatures', () => {
        expect(signatureCanBeConnection(signature({}))).toBe(true);
    });

    it('rejects site categories', () => {
        for (const categoryCode of ['gas', 'data', 'relic', 'combat', 'ore']) {
            expect(signatureCanBeConnection(signature({ categoryCode }))).toBe(false);
        }
    });
});

describe('signatureCanLeadToClass', () => {
    it('accepts signatures without a resolved type', () => {
        expect(signatureCanLeadToClass(signature({ hasType: false }), '4')).toBe(true);
    });

    it('accepts wormhole types leading to the target class', () => {
        expect(signatureCanLeadToClass(signature({ targetClass: '4' }), '4')).toBe(true);
    });

    it('rejects wormhole types leading to a different class', () => {
        expect(signatureCanLeadToClass(signature({ targetClass: 'n' }), '4')).toBe(false);
    });

    it('accepts wormhole types with an unknown destination', () => {
        expect(signatureCanLeadToClass(signature({ targetClass: 'unknown' }), '4')).toBe(true);
    });

    it('accepts typed wormholes when the target class is not known', () => {
        expect(signatureCanLeadToClass(signature({ targetClass: 'n' }), null)).toBe(true);
    });

    it('matches known-space classes exactly', () => {
        expect(signatureCanLeadToClass(signature({ targetClass: 'h' }), 'h')).toBe(true);
        expect(signatureCanLeadToClass(signature({ targetClass: 'h' }), 'l')).toBe(false);
    });
});

describe('groupSignatureOptions', () => {
    it('splits signatures into likely, connected and unlikely sections', () => {
        const matching = signature({ id: 1, categoryCode: 'wormhole', targetClass: '4' });
        const unscanned = signature({ id: 2 });
        const connected = signature({ id: 3, categoryCode: 'wormhole', targetClass: '4', connected: true });
        const wrongClass = signature({ id: 4, categoryCode: 'wormhole', targetClass: 'n' });
        const site = signature({ id: 5, categoryCode: 'gas' });

        const groups = groupSignatureOptions([matching, unscanned, connected, wrongClass, site], '4');

        expect(groups.likely).toEqual([matching, unscanned]);
        expect(groups.connected).toEqual([connected]);
        expect(groups.unlikely).toEqual([wrongClass]);
    });

    it('puts connected signatures into the connected section even when their class cannot match', () => {
        const connectedWrongClass = signature({ id: 1, categoryCode: 'wormhole', targetClass: 'n', connected: true });

        const groups = groupSignatureOptions([connectedWrongClass], '4');

        expect(groups.connected).toEqual([connectedWrongClass]);
        expect(groups.unlikely).toEqual([]);
    });

    it('drops site signatures entirely', () => {
        const groups = groupSignatureOptions([signature({ id: 1, categoryCode: 'relic' })], '4');

        expect(groups).toEqual({ likely: [], connected: [], unlikely: [] });
    });
});
