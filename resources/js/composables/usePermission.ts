import { AppPageProps } from '@/types';
import { usePage } from '@inertiajs/vue3';
import { computed } from 'vue';

type PermissionLevel = 'viewer' | 'member' | 'manager';

const levels: PermissionLevel[] = ['viewer', 'member', 'manager'];

export default function usePermission() {
    const page = usePage<
        AppPageProps<{
            permission?: PermissionLevel | null;
        }>
    >();

    const permission = computed(() => page.props.permission ?? null);

    function isAtLeast(level: PermissionLevel): boolean {
        const currentPermission = permission.value;
        if (currentPermission === null) return false;
        const current = levels.indexOf(currentPermission);
        const required = levels.indexOf(level);
        return current >= required;
    }

    const canEdit = computed(() => isAtLeast('member'));
    const canManageAccess = computed(() => isAtLeast('manager'));
    const isViewer = computed(() => permission.value === 'viewer');

    return {
        permission,
        isAtLeast,
        canEdit,
        canManageAccess,
        isViewer,
    };
}
