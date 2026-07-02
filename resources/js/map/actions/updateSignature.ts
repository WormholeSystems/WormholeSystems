import Signatures from '@/routes/signatures';
import { TSignature } from '@/types/models';
import type { FormDataConvertible } from '@inertiajs/core';
import { router } from '@inertiajs/vue3';

export function updateSignature(signature: TSignature, data: Record<string, FormDataConvertible>): void {
    return router.put(Signatures.update(signature.id).url, data, {
        preserveScroll: true,
        preserveState: true,
        only: ['map', 'selected_map_solarsystem'],
        onError: () => router.reload({ only: ['map'] }),
    });
}
