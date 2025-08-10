import SignatureController from '@/routes/signatures';
import { router } from '@inertiajs/vue3';

export function createSignature(map_solarsystem_id: number) {
    return router.post(
        SignatureController.store().url,
        {
            map_solarsystem_id,
            signature_id: '',
            type: null,
            category: null,
        },
        {
            preserveScroll: true,
            preserveState: true,
            only: ['selected_map_solarsystem'],
        },
    );
}
