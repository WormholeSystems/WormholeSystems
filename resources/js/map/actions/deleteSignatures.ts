import Signatures from '@/routes/map-solarsystems/signatures';
import { router } from '@inertiajs/vue3';

export function deleteSignatures(map_solarsystem_id: number, signature_ids: number[], with_solarsystems: boolean = false): void {
    return router.delete(Signatures.destroy(map_solarsystem_id).url, {
        preserveScroll: true,
        preserveState: true,
        only: ['selected_map_solarsystem'],
        data: {
            signature_ids,
            remove_map_solarsystems: with_solarsystems,
        },
        onError: () => router.reload({ only: ['map'] }),
    });
}
