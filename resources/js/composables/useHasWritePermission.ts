import usePermission from '@/composables/usePermission';

/**
 * @deprecated Use usePermission().canEdit instead
 */
export default function useHasWritePermission() {
    const { canEdit } = usePermission();
    return canEdit;
}
