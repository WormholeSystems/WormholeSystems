import { signatureCategories, signatureTypes } from '@/const/signatures';
import { signatureParser } from '@/lib/SignatureParser';
import { describe, expect, it, vi } from 'vitest';

vi.mock('vue-sonner', () => ({ toast: { error: vi.fn() } }));

const factionWarfare = signatureCategories.find((cat) => cat.name === 'Factional Warfare Site')!;
const gas = signatureCategories.find((cat) => cat.name === 'Gas Site')!;
const ore = signatureCategories.find((cat) => cat.name === 'Ore Site')!;

describe('parseSignatures', () => {
    it('parses a signature with a known category and type', () => {
        const type = signatureTypes.find((t) => t.signature_category_id === gas.id)!;
        const line = `ABC-123\tCosmic Signature\tGas Site\t${type.name}\t100,0%\t5,03 AU`;

        const [parsed] = signatureParser.parseSignatures(line);

        expect(parsed).toMatchObject({
            signature_id: 'ABC-123',
            signature_category_id: gas.id,
            signature_type_id: type.id,
            raw_type_name: null,
        });
    });

    it('parses faction warfare sites with the raw type name', () => {
        const line = 'BUH-704\tCosmic Anomaly\tFactional Warfare Site - Combat Site\tMinmatar Large NVY-1\t100,0%\t13,83 AU';

        const [parsed] = signatureParser.parseSignatures(line);

        expect(parsed).toMatchObject({
            signature_id: 'BUH-704',
            signature_category_id: factionWarfare.id,
            signature_type_id: null,
            raw_type_name: 'Minmatar Large NVY-1',
        });
    });

    it('parses a real scan window paste with faction warfare, ore, and unscanned rows', () => {
        const paste = [
            'BBL-893\tCosmic Anomaly\tFactional Warfare Site - Combat Site\tAmarr Scout BSC-1\t100,0%\t6,84 AU',
            'CBA-620\tCosmic Anomaly\tOre Site\tGlacial Mass Belt\t100,0%\t7,02 AU',
            'DXA-556\tCosmic Signature\t\t\t0,0%\t6,51 AU',
            'MSA-264\tCosmic Anomaly\tFactional Warfare Site - Combat Site\tAmarr Moderate NVY-3\t100,0%\t60,60 AU',
        ].join('\n');

        const parsed = signatureParser.parseSignatures(paste);

        expect(parsed).toHaveLength(4);
        expect(parsed[0]).toMatchObject({
            signature_id: 'BBL-893',
            signature_category_id: factionWarfare.id,
            raw_type_name: 'Amarr Scout BSC-1',
        });
        expect(parsed[1]).toMatchObject({
            signature_id: 'CBA-620',
            signature_category_id: ore.id,
            raw_type_name: 'Glacial Mass Belt',
        });
        expect(parsed[2]).toMatchObject({
            signature_id: 'DXA-556',
            signature_category_id: null,
            signature_type_id: null,
            raw_type_name: null,
        });
        expect(parsed[3]).toMatchObject({
            signature_id: 'MSA-264',
            signature_category_id: factionWarfare.id,
            raw_type_name: 'Amarr Moderate NVY-3',
        });
    });

    it('leaves unknown categories uncategorised', () => {
        const line = 'XYZ-999\tCosmic Signature\tSomething Unheard Of\tMystery Site\t12,5%\t1,00 AU';

        const [parsed] = signatureParser.parseSignatures(line);

        expect(parsed).toMatchObject({
            signature_id: 'XYZ-999',
            signature_category_id: null,
            signature_type_id: null,
            raw_type_name: null,
        });
    });
});
