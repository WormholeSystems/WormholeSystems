import { TRawSignature } from '@/lib/SignatureParser';
import PasteSignatures from '@/routes/paste-signatures';
import { router } from '@inertiajs/vue3';

export function pasteSignatures(map_solarsystem_id: number, signatures: TRawSignature[]) {
    return router.post(
        PasteSignatures.store().url,
        {
            map_solarsystem_id,
            signatures: signatures.map((signature) => ({
                signature_id: signature.signature_id,
                signature_category_id: signature.signature_category_id,
                signature_type_id: signature.signature_type_id,
                raw_type_name: !signature.signature_type_id ? signature.raw_type_name : undefined,
            })),
        },
        {
            preserveScroll: true,
            preserveState: true,
            only: ['selected_map_solarsystem', 'map'],
        },
    );
}
