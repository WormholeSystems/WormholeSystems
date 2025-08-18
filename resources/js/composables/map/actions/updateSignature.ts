import Signatures from '@/routes/signatures';
import { TSignature } from '@/types/models';
import { router } from '@inertiajs/vue3';

export function updateSignature(signature: TSignature, data: Record<string, any>) {
    return router.put(Signatures.update(signature.id).url, data, {
        preserveScroll: true,
        preserveState: true,
        only: ['selected_map_solarsystem', 'map'],
    });
}
