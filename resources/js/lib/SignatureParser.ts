import { signatureCategories, signatureTypes } from '@/const/signatures';
import { TSignatureCategory, TSignatureType } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { toast } from 'vue-sonner';

export type TRawSignature = {
    signature_id: string;
    signature_category_id: number | null;
    signature_type_id: number | null;
    raw_type_name: string | null;
    created_at?: string;
};

class SignatureParser {
    parseSignatures(text: string): TRawSignature[] {
        if (!text) {
            return [] satisfies TRawSignature[];
        }

        return text
            .split('\n')
            .map((sig) => sig.split('\t'))
            .map((sig) => this.parseSignature(sig))
            .filter((sig): sig is TRawSignature => sig !== null);
    }

    parseSignature(signature: string[]): TRawSignature | null {
        if (signature.length < 4) {
            toast.error('Invalid signature format. Expected at least 4 tab-separated values.');
            return null;
        }

        const [signature_id, _, category_name, type_name] = signature;

        if (!signature_id) {
            toast.error('Invalid signature format. Signature ID is required.');
            return null;
        }

        const signature_category = this.getCategory(category_name);
        const signature_type = this.getType(signature_category, type_name);

        // Store the raw type name if we have a category but no matching type
        // This captures temporary event sites that aren't in the database
        const raw_type_name = signature_category && !signature_type && type_name?.trim() ? type_name.trim() : null;

        return {
            signature_id: signature_id.trim(),
            signature_category_id: signature_category?.id || null,
            signature_type_id: signature_type?.id || null,
            raw_type_name,
            created_at: new UTCDate().toISOString(),
        };
    }

    getCategory(categoryName: string): TSignatureCategory | null {
        return signatureCategories.find((cat) => cat.name === categoryName) || null;
    }

    getType(category: TSignatureCategory | null, typeName: string): TSignatureType | null {
        if (!category) {
            return null;
        }
        if (category.name === 'Wormhole') {
            return null;
        }

        return signatureTypes.find((type) => type.name === typeName.trim() && type.signature_category_id === category.id) || null;
    }
}

export const signatureParser = new SignatureParser();
