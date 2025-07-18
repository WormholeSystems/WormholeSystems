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
            toast.error('Invalid signature format. Expected at least 4 tab-separated values.');
            return null;
        }

        const [signature_id, _, category, type] = signature;

        let app_category: string | null = category;

        if (!signature_id) {
            toast.error('Invalid signature format. Signature ID is required.');
            return null;
        }

        if (!signatureCategories.includes(category as TSignatureCategory)) {
            app_category = null;
        }

        function getType(category: string | null, type: string | null) {
            if (category === 'Wormhole') {
                return null;
            }
            const parsed_type = type?.trim();
            if (!parsed_type) {
                return null;
            }

            return parsed_type;
        }

        return {
            signature_id: signature_id.trim(),
            category: app_category as TSignatureCategory,
            type: getType(app_category, type),
            created_at: new Date().toISOString(),
        };
    }
}

export const signatureParser = new SignatureParser();
