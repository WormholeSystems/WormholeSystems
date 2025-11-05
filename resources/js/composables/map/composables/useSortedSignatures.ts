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
