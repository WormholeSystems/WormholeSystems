import { buildSignatureBookmark, formatBookmarkName, isReturnBookmark, TBookmarkFormats } from '@/lib/bookmark';
import { describe, expect, it } from 'vitest';

const NUMERIC_FORMATS: TBookmarkFormats = {};
const ALPHABETICAL_FORMATS: TBookmarkFormats & { bookmark_alias_scheme: 'alphabetical' } = { bookmark_alias_scheme: 'alphabetical' };

function baseSignature(overrides: Partial<Parameters<typeof buildSignatureBookmark>[0]['signature']> = {}) {
    return {
        signature_id: 'ABC-123',
        ship_size: null,
        mass_status: null,
        lifetime: 'healthy' as const,
        wormhole: null,
        signature_type: null,
        ...overrides,
    };
}

describe('buildSignatureBookmark (connected)', () => {
    it('uses the real target alias, class and name', () => {
        const name = buildSignatureBookmark({
            signature: baseSignature(),
            currentSystem: { alias: 'HOME' },
            connectionTarget: {
                alias: 'AC',
                occupier_alias: null,
                solarsystem: { class: 'h', name: 'Jita', region: { name: 'The Forge' } },
            },
            aliases: [],
            formats: { ...NUMERIC_FORMATS, bookmark_format_kspace: '{alias} {class} {name} {region}' },
        });

        expect(name).toBe('AC HS Jita The Forge');
    });

    it('falls back to the guessed alias when the target has none yet', () => {
        const name = buildSignatureBookmark({
            signature: baseSignature(),
            currentSystem: { alias: 'A' },
            connectionTarget: {
                alias: null,
                occupier_alias: null,
                solarsystem: { class: '3', name: 'J123456' },
            },
            aliases: [],
            formats: { ...NUMERIC_FORMATS, bookmark_format_wormhole: '{alias} {sig} {class}' },
        });

        expect(name).toBe('A1 ABC C3');
    });

    it('picks a reserved-letter guess for the fallback under the alphabetical scheme', () => {
        const name = buildSignatureBookmark({
            signature: baseSignature(),
            currentSystem: { alias: 'A' },
            connectionTarget: {
                alias: '',
                occupier_alias: null,
                solarsystem: { class: 'l', name: 'Amamake' },
            },
            aliases: [],
            formats: { ...ALPHABETICAL_FORMATS, bookmark_format_kspace: '{alias} {class}' },
        });

        expect(name).toBe('AL1 LS');
    });

    it('matches formatBookmarkName for an already-aliased target', () => {
        const connectionTarget = { alias: 'ZZ', occupier_alias: 'Occupier', solarsystem: { class: '3' as const, name: 'J1' } };
        const context = { signatureId: 'ABC-123', shipSize: null, massStatus: null, lifetime: 'healthy' as const, wormholeCode: null };

        const viaBuilder = buildSignatureBookmark({
            signature: baseSignature(),
            currentSystem: { alias: 'HOME' },
            connectionTarget,
            aliases: [],
            formats: NUMERIC_FORMATS,
        });

        expect(viaBuilder).toBe(formatBookmarkName(connectionTarget, context, NUMERIC_FORMATS));
    });
});

describe('buildSignatureBookmark (unconnected, numeric)', () => {
    it('numbers the next child under the current alias', () => {
        const name = buildSignatureBookmark({
            signature: baseSignature(),
            currentSystem: { alias: '1' },
            connectionTarget: null,
            aliases: [],
            formats: NUMERIC_FORMATS,
        });

        expect(name).toBe('11 ABC');
    });

    it('numbers from 1 for an unaliased current system', () => {
        const name = buildSignatureBookmark({
            signature: baseSignature(),
            currentSystem: { alias: null },
            connectionTarget: null,
            aliases: [],
            formats: NUMERIC_FORMATS,
        });

        expect(name).toBe('1 ABC');
    });
});

describe('buildSignatureBookmark (unconnected, alphabetical)', () => {
    it('letters the next wormhole child, skipping H/L/N/P', () => {
        const name = buildSignatureBookmark({
            signature: baseSignature({ signature_type: { target_class: '3' } }),
            currentSystem: { alias: 'A' },
            connectionTarget: null,
            aliases: ['AA', 'AB', 'AC', 'AD', 'AE', 'AF', 'AG'],
            formats: ALPHABETICAL_FORMATS,
        });

        expect(name).toBe('AI ABC C3');
    });

    it('assigns a reserved letter and per-type index for a k-space target', () => {
        const name = buildSignatureBookmark({
            signature: baseSignature({ signature_type: { target_class: 'h' } }),
            currentSystem: { alias: 'A' },
            connectionTarget: null,
            aliases: [],
            formats: ALPHABETICAL_FORMATS,
        });

        expect(name).toBe('AH1 HS ABC');
    });

    it('reserves N for a null-sec target', () => {
        const name = buildSignatureBookmark({
            signature: baseSignature({ signature_type: { target_class: 'n' } }),
            currentSystem: { alias: 'B' },
            connectionTarget: null,
            aliases: ['BN1'],
            formats: ALPHABETICAL_FORMATS,
        });

        expect(name).toBe('BN2 NS ABC');
    });

    it('reserves P for a Pochven target', () => {
        const name = buildSignatureBookmark({
            signature: baseSignature({ signature_type: { target_class: 'p' } }),
            currentSystem: { alias: 'A' },
            connectionTarget: null,
            aliases: [],
            formats: ALPHABETICAL_FORMATS,
        });

        expect(name).toBe('AP1 P ABC');
    });

    it('treats an unidentified type as a wormhole for the guess and leaves the class blank', () => {
        const name = buildSignatureBookmark({
            signature: baseSignature({ signature_type: { target_class: null } }),
            currentSystem: { alias: 'A' },
            connectionTarget: null,
            aliases: ['AA'],
            formats: ALPHABETICAL_FORMATS,
        });

        // Next plain letter (B), not a reserved k-space letter, and no "UNKNOWN" class token.
        expect(name).toBe('AB ABC');
    });

    it('treats "unknown" the same as a missing target class', () => {
        const name = buildSignatureBookmark({
            signature: baseSignature({ signature_type: { target_class: 'unknown' } }),
            currentSystem: { alias: 'A' },
            connectionTarget: null,
            aliases: ['AA'],
            formats: ALPHABETICAL_FORMATS,
        });

        expect(name).toBe('AB ABC');
    });
});

describe('buildSignatureBookmark (class token)', () => {
    it('renders wormhole classes as "C<n>"', () => {
        const name = buildSignatureBookmark({
            signature: baseSignature({ signature_type: { target_class: '5' } }),
            currentSystem: { alias: null },
            connectionTarget: null,
            aliases: [],
            formats: NUMERIC_FORMATS,
        });

        expect(name).toBe('1 ABC C5');
    });

    it.each([
        ['h', 'HS'],
        ['l', 'LS'],
        ['n', 'NS'],
    ])('renders k-space class "%s" as "%s"', (targetClass, label) => {
        const name = buildSignatureBookmark({
            signature: baseSignature({ signature_type: { target_class: targetClass } }),
            currentSystem: { alias: null },
            connectionTarget: null,
            aliases: [],
            formats: { ...NUMERIC_FORMATS, bookmark_format_kspace: '{alias} {sig} {class}' },
        });

        expect(name).toBe(`1 ABC ${label}`);
    });
});

describe('isReturnBookmark', () => {
    it('is true when the destination alias is a substring of the opposite alias (up-chain)', () => {
        expect(isReturnBookmark('A', 'AB', null)).toBe(true);
    });

    it('is false when the opposite alias is a substring of the destination (down-chain)', () => {
        expect(isReturnBookmark('AB', 'A', null)).toBe(false);
    });

    it('is false for a sibling branch whose alias is a non-prefix substring of the opposite alias', () => {
        expect(isReturnBookmark('B', 'AB', null)).toBe(false);
        expect(isReturnBookmark('1', '21', null)).toBe(false);
    });

    it('is true when the destination alias is the ignored alias, regardless of the opposite alias', () => {
        expect(isReturnBookmark('HOME', 'AB', 'HOME')).toBe(true);
    });

    it('is false when either alias is empty', () => {
        expect(isReturnBookmark('', 'AB', null)).toBe(false);
        expect(isReturnBookmark('A', '', null)).toBe(false);
        expect(isReturnBookmark(null, null, null)).toBe(false);
    });

    it('is false for an ignored-alias destination when the opposite alias is omitted', () => {
        expect(isReturnBookmark('HOME', null, 'HOME')).toBe(false);
        expect(isReturnBookmark('HOME', '', 'HOME')).toBe(false);
    });
});

describe('formatBookmarkName (oppositeAlias / return format)', () => {
    const system = { alias: 'A', solarsystem: { class: '3' as const, name: 'J123456' } };
    const context = { signatureId: 'ABC-123', shipSize: null, massStatus: null, lifetime: 'healthy' as const, wormholeCode: null };
    const formats: TBookmarkFormats = { bookmark_format_wormhole: '{alias} {sig} {class}', bookmark_format_return: '*{alias} {sig} {class}' };

    it('renders the return template when the destination is up-chain of the opposite alias', () => {
        expect(formatBookmarkName(system, context, formats, 'AB')).toBe('*A ABC C3');
    });

    it('renders the return template when the destination is the ignored alias', () => {
        const home = { ...system, alias: 'HOME' };
        expect(formatBookmarkName(home, context, { ...formats, bookmark_ignored_alias: 'HOME' }, 'AB')).toBe('*HOME ABC C3');
    });

    it('renders the forward wormhole/k-space template when not a return', () => {
        expect(formatBookmarkName(system, context, formats, 'B')).toBe('A ABC C3');
    });

    it('never selects the return format when oppositeAlias is omitted (legacy callers)', () => {
        expect(formatBookmarkName(system, context, formats)).toBe('A ABC C3');
    });
});

describe('buildSignatureBookmark (detectReturn)', () => {
    it('renders the return template for an up-chain connected target when detectReturn is true', () => {
        const name = buildSignatureBookmark({
            signature: baseSignature(),
            currentSystem: { alias: 'AB' },
            connectionTarget: { alias: 'A', occupier_alias: null, solarsystem: { class: '3', name: 'J1' } },
            aliases: [],
            formats: { ...NUMERIC_FORMATS, bookmark_format_return: '*{alias} {sig} {class}' },
            detectReturn: true,
        });

        expect(name).toBe('*A ABC C3');
    });

    it('renders the return template for a connected target named the ignored (home) alias when detectReturn is true', () => {
        const name = buildSignatureBookmark({
            signature: baseSignature(),
            currentSystem: { alias: 'AB' },
            connectionTarget: { alias: 'HOME', occupier_alias: null, solarsystem: { class: '3', name: 'J1' } },
            aliases: [],
            formats: { ...NUMERIC_FORMATS, bookmark_format_return: '*{alias} {sig} {class}', bookmark_ignored_alias: 'HOME' },
            detectReturn: true,
        });

        expect(name).toBe('*HOME ABC C3');
    });

    it('renders the normal forward template for a down-chain target when detectReturn is true', () => {
        const name = buildSignatureBookmark({
            signature: baseSignature(),
            currentSystem: { alias: 'A' },
            connectionTarget: { alias: 'AB', occupier_alias: null, solarsystem: { class: '3', name: 'J1' } },
            aliases: [],
            formats: { ...NUMERIC_FORMATS, bookmark_format_return: '*{alias} {sig} {class}' },
            detectReturn: true,
        });

        expect(name).toBe('AB ABC C3');
    });

    it('stays forward for an up-chain / home target when detectReturn is omitted (the auto-copy path)', () => {
        const name = buildSignatureBookmark({
            signature: baseSignature(),
            currentSystem: { alias: 'AB' },
            connectionTarget: { alias: 'HOME', occupier_alias: null, solarsystem: { class: '3', name: 'J1' } },
            aliases: [],
            formats: { ...NUMERIC_FORMATS, bookmark_format_return: '*{alias} {sig} {class}', bookmark_ignored_alias: 'HOME' },
        });

        expect(name).toBe('HOME ABC C3');
    });

    it('stays forward for an up-chain target when detectReturn is explicitly false', () => {
        const name = buildSignatureBookmark({
            signature: baseSignature(),
            currentSystem: { alias: 'AB' },
            connectionTarget: { alias: 'A', occupier_alias: null, solarsystem: { class: '3', name: 'J1' } },
            aliases: [],
            formats: { ...NUMERIC_FORMATS, bookmark_format_return: '*{alias} {sig} {class}' },
            detectReturn: false,
        });

        expect(name).toBe('A ABC C3');
    });
});
