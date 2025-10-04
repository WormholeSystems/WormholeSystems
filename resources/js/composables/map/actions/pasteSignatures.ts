import { TRawSignature } from '@/lib/SignatureParser';
import { getCategoryByName, getTypesByCategoryName } from '@/const/signatures';
import PasteSignatures from '@/routes/paste-signatures';
import { router } from '@inertiajs/vue3';

export function pasteSignatures(map_solarsystem_id: number, signatures: TRawSignature[]) {
    return router.post(
        PasteSignatures.store().url,
        {
            map_solarsystem_id,
            signatures: signatures.map((signature) => {
                // Translate category string to ID
                const category = signature.category ? getCategoryByName(signature.category) : null;
                const signature_category_id = category?.id ?? null;
                
                // Translate type string to ID
                let signature_type_id: number | null = null;
                if (signature.type && signature.category) {
                    const types = getTypesByCategoryName(signature.category);
                    const type = types.find(t => t.name === signature.type);
                    signature_type_id = type?.id ?? null;
                }
                
                return {
                    signature_id: signature.signature_id,
                    signature_category_id,
                    signature_type_id,
                };
            }),
        },
        {
            preserveScroll: true,
            preserveState: true,
            only: ['selected_map_solarsystem', 'map'],
        },
    );
}
