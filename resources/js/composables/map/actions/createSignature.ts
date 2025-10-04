import MapSolarsystems from '@/routes/map-solarsystems';
import { router } from '@inertiajs/vue3';

export function createSignature(map_solarsystem_id: number) {
    return router.post(
        MapSolarsystems.signatures.store(map_solarsystem_id),
        {
            map_solarsystem_id,
            signature_id: '',
            signature_type_id: null,
            signature_category_id: null,
        },
        {
            preserveScroll: true,
            preserveState: true,
            only: ['selected_map_solarsystem'],
        },
    );
}
