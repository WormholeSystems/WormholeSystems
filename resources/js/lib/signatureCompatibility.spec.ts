import { signatureCanLeadToClass } from '@/lib/signatureCompatibility';
import type { TSignature, TSignatureCategory, TSignatureType, TStringedSolarsystemClass } from '@/types/models';
import { describe, expect, it } from 'vitest';

function signature(overrides: { categoryCode?: string; targetClass?: TStringedSolarsystemClass | 'unknown' | null; hasType?: boolean }): TSignature {
    const { categoryCode, targetClass = null, hasType = targetClass !== null } = overrides;

    return {
        id: 1,
        signature_id: 'ABC-123',
        signature_category: categoryCode ? ({ id: 1, name: categoryCode, code: categoryCode } as TSignatureCategory) : null,
        signature_type: hasType ? ({ id: 1, name: 'Test', signature: 'X702', target_class: targetClass } as TSignatureType) : null,
    } as TSignature;
}

describe('signatureCanLeadToClass', () => {
    it('accepts signatures without any category or type', () => {
        expect(signatureCanLeadToClass(signature({}), '4')).toBe(true);
    });

    it('rejects signatures categorised as non-wormhole sites', () => {
        expect(signatureCanLeadToClass(signature({ categoryCode: 'gas' }), '4')).toBe(false);
    });

    it('accepts wormhole types leading to the target class', () => {
        expect(signatureCanLeadToClass(signature({ categoryCode: 'wormhole', targetClass: '4' }), '4')).toBe(true);
    });

    it('rejects wormhole types leading to a different class', () => {
        expect(signatureCanLeadToClass(signature({ categoryCode: 'wormhole', targetClass: 'n' }), '4')).toBe(false);
    });

    it('accepts wormhole types with an unknown destination', () => {
        expect(signatureCanLeadToClass(signature({ categoryCode: 'wormhole', targetClass: 'unknown' }), '4')).toBe(true);
    });

    it('accepts typed wormholes when the target class is not known', () => {
        expect(signatureCanLeadToClass(signature({ categoryCode: 'wormhole', targetClass: 'n' }), null)).toBe(true);
    });

    it('accepts wormhole-category signatures without a resolved type', () => {
        expect(signatureCanLeadToClass(signature({ categoryCode: 'wormhole', hasType: false }), 'h')).toBe(true);
    });

    it('matches known-space classes exactly', () => {
        expect(signatureCanLeadToClass(signature({ categoryCode: 'wormhole', targetClass: 'h' }), 'h')).toBe(true);
        expect(signatureCanLeadToClass(signature({ categoryCode: 'wormhole', targetClass: 'h' }), 'l')).toBe(false);
    });
});
