import { signatureCategories, signatureTypes } from '@/const/signatures';
import { signatureParser } from '@/lib/SignatureParser';
import { describe, expect, it, vi } from 'vitest';

vi.mock('vue-sonner', () => ({ toast: { error: vi.fn() } }));

const factionWarfare = signatureCategories.find((cat) => cat.name === 'Factional Warfare Site')!;
const gas = signatureCategories.find((cat) => cat.name === 'Gas Site')!;
const ore = signatureCategories.find((cat) => cat.name === 'Ore Site')!;
const combat = signatureCategories.find((cat) => cat.name === 'Combat Site')!;
const data = signatureCategories.find((cat) => cat.name === 'Data Site')!;
const relic = signatureCategories.find((cat) => cat.name === 'Relic Site')!;
const wormhole = signatureCategories.find((cat) => cat.name === 'Wormhole')!;
const typeNamed = (name: string) => signatureTypes.find((type) => type.name === name)!;

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

    describe('localized clients', () => {
        it('parses a German paste with decimal commas', () => {
            const paste = [
                'ABC-123\tKosmische Signatur\tDatengebiet\tUngesicherter Verstärker im Außenbereich\t100,0%\t4,2 AE',
                'DEF-456\tKosmische Signatur\tWurmloch\tInstabiles Wurmloch\t100,0%\t12,34 AE',
                'GHI-789\tKosmische Signatur\t\t\t0,0%\t6,51 AE',
            ].join('\n');

            const parsed = signatureParser.parseSignatures(paste);

            expect(parsed).toHaveLength(3);
            expect(parsed[0]).toMatchObject({
                signature_id: 'ABC-123',
                signature_category_id: data.id,
                signature_type_id: typeNamed('Unsecured Perimeter Amplifier').id,
                raw_type_name: null,
            });
            expect(parsed[1]).toMatchObject({ signature_id: 'DEF-456', signature_category_id: wormhole.id, signature_type_id: null });
            expect(parsed[2]).toMatchObject({ signature_id: 'GHI-789', signature_category_id: null, signature_type_id: null });
        });

        it('parses Russian and Chinese pastes', () => {
            const paste = [
                'ABC-123\tСкрытый сигнал\tАрхеологический район\tРазрушенный храмовый комплекс Sansha\t100,0%\t3,10 а.е.',
                'DEF-456\t空间信号\t战斗地点\t核心兵站\t100.0%\t5.03 AU',
            ].join('\n');

            const parsed = signatureParser.parseSignatures(paste);

            expect(parsed[0]).toMatchObject({
                signature_category_id: relic.id,
                signature_type_id: typeNamed('Ruined Sansha Temple Site').id,
                raw_type_name: null,
            });
            expect(parsed[1]).toMatchObject({
                signature_category_id: combat.id,
                signature_type_id: typeNamed('Core Garrison').id,
                raw_type_name: null,
            });
        });

        it('parses a clipboard mixing several languages', () => {
            const gasType = typeNamed('Barren Perimeter Reservoir');
            const paste = [
                `AAA-111\tCosmic Signature\tGas Site\t${gasType.name}\t100.0%\t5.03 AU`,
                'BBB-222\tKosmische Signatur\tGasgebiet\tKarges Gasvorkommen\t100,0%\t4,2 AE',
                'CCC-333\t코즈믹 시그니처\t가스 사이트\t황량한 변방 가스 매장지\t100.0%\t1.20 AU',
                'DDD-444\t宇宙のシグネチャ\tガスサイト\t荒地の防御ライン貯水池\t100.0%\t2.40 AU',
                'EEE-555\tSignature cosmique\tSite de collecte de gaz\tRéservoir de périmètre aride\t100,0%\t7,00 UA',
                'FFF-666\tSeñal cósmica\tZona de gas\tReserva perimetral baldía\t100,0%\t8,00 UA',
            ].join('\n');

            const parsed = signatureParser.parseSignatures(paste);

            expect(parsed).toHaveLength(6);
            parsed.forEach((signature) => {
                expect(signature).toMatchObject({
                    signature_category_id: gas.id,
                    signature_type_id: gasType.id,
                    raw_type_name: null,
                });
            });
        });

        it('recognizes localized faction warfare sites', () => {
            const line =
                'BUH-704\tКосмическая аномалия\tРайон прохождения межгосударственных войн - Боевой район\tМинматарский объект\t100,0%\t13,83 а.е.';

            const [parsed] = signatureParser.parseSignatures(line);

            expect(parsed).toMatchObject({
                signature_category_id: factionWarfare.id,
                signature_type_id: null,
                raw_type_name: 'Минматарский объект',
            });
        });

        it('treats non-breaking spaces like regular spaces', () => {
            const line = 'ABC-123\tКосмическая аномалия\tБоевой\u00a0район\tНеустойчивая\u00a0червоточина\t100,0%\t1,00 а.е.';

            const [parsed] = signatureParser.parseSignatures(line);

            expect(parsed).toMatchObject({
                signature_category_id: combat.id,
                signature_type_id: typeNamed('Unstable Wormhole').id,
            });
        });

        it('keeps the raw name when a translation is shared by several sites', () => {
            const line = 'ABC-123\tKosmische Anomalie\tMineraliengebiet\tGewöhnliche Erzvorkommen im Grenzgebiet\t100,0%\t1,00 AE';

            const [parsed] = signatureParser.parseSignatures(line);

            expect(parsed).toMatchObject({
                signature_category_id: ore.id,
                signature_type_id: null,
                raw_type_name: 'Gewöhnliche Erzvorkommen im Grenzgebiet',
            });
        });
    });
});
