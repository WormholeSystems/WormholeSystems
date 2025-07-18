import { toast } from 'vue-sonner';

export type TSignatureCategory = 'Wormhole' | 'Data Site' | 'Relic Site' | 'Combat Site' | 'Gas Site' | 'Ore Site' | null;
export type TRawSignature = {
    signature_id: string;
    category: TSignatureCategory;
    type: string | null;
    created_at?: string;
};

export const signatureCategories = ['Wormhole', 'Data Site', 'Relic Site', 'Combat Site', 'Gas Site', 'Ore Site', null] as const;

class SignatureParser {
    parseSignatures(text: string): TRawSignature[] {
        if (!text) {
            return [] satisfies TRawSignature[];
        }

        return text
            .split('\n')
            .map((sig) => sig.split('\t'))
            .map(this.parseSignature)
            .filter((sig): sig is TRawSignature => sig !== null);
    }

    parseSignature(signature: string[]): TRawSignature | null {
        if (signature.length < 4) {
            return null;
        }

        console.log(`Parsing signature: ${signature.join(', ')}`);

        const [signature_id, _, category, type] = signature;

        let app_category: string | null = category;

        if (!signature_id) {
            toast.error('Signature ID is missing');
            return null;
        }

        if (!signatureCategories.includes(category as TSignatureCategory)) {
            app_category = null;
            return null;
        }

        return {
            signature_id: signature_id.trim(),
            category: app_category as TSignatureCategory,
            type: category === 'Wormhole' ? null : type.trim(),
            created_at: new Date().toISOString(), // Default to current time if not provided
        };
    }
}

export const signatureParser = new SignatureParser();
