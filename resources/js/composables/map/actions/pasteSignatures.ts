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
                type: signature.type,
                category: signature.category,
            })),
        },
        {
            preserveScroll: true,
            preserveState: true,
            only: ['selected_map_solarsystem'],
        },
    );
}
