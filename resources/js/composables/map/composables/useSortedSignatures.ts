import { useSortPreference } from '@/composables/useSortPreference';
import { TSignature } from '@/types/models';
import { computed, Ref } from 'vue';

export function useSortableSignatures<T extends TSignature>(signatures: Ref<T[]>) {
    const { sortPreferences, updateSortPreferences } = useSortPreference('signatures');
    const sorted = computed(() => signatures.value.sort(getSortComparison));

    function compareNullableStrings(a: string | null, b: string | null): number {
        if (!a && !b) return 0;

        if (!a) return 1;

        if (!b) return -1;

        return a.localeCompare(b);
    }

    function getModifiedDate(signature: TSignature): string {
        // For wormholes, use created_at; for others, use updated_at
        if (signature.signature_category?.name === 'Wormhole') {
            return signature.created_at;
        }
        return signature.updated_at;
    }

    function getSortComparison(a: TSignature, b: TSignature): number {
        let primaryComparison: number;

        switch (sortPreferences.value.column) {
            case 'id':
                primaryComparison = compareNullableStrings(a.signature_id, b.signature_id);
                break;
            case 'category':
                primaryComparison = compareNullableStrings(a.signature_category?.name ?? null, b.signature_category?.name ?? null);
                break;
            case 'type':
                primaryComparison = compareNullableStrings(a.signature_type?.name ?? null, b.signature_type?.name ?? null);
                break;
            case 'age':
                // Compare dates - newer signatures should be "less than" older ones for ascending sort
                const aDate = new Date(getModifiedDate(a)).getTime();
                const bDate = new Date(getModifiedDate(b)).getTime();
                primaryComparison = bDate - aDate; // Reverse comparison so newer = smaller (comes first in asc)
                break;
            default:
                primaryComparison = 0;
        }

        const directedPrimaryComparison = sortPreferences.value.direction === 'desc' ? -primaryComparison : primaryComparison;

        if (primaryComparison === 0) {
            return compareNullableStrings(a.signature_id, b.signature_id);
        }

        return directedPrimaryComparison;
    }

    return {
        sorted,
        sortPreferences,
        updateSortPreferences,
    };
}
