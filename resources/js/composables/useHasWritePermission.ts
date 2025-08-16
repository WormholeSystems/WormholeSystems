import { AppPageProps } from '@/types';
import { usePage } from '@inertiajs/vue3';
import { computed } from 'vue';

export default function useHasWritePermission() {
    const page = usePage<
        AppPageProps<{
            has_write_access?: boolean;
        }>
    >();

    return computed(() => {
        return page.props.has_write_access === true;
    });
}
