import { buildSignatureBookmark, formatBookmarkName, TBookmarkFormats } from '@/lib/bookmark';
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
