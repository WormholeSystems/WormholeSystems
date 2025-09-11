import { setCookie } from '@/lib/utils';
import { AppPageProps } from '@/types';
import { usePage } from '@inertiajs/vue3';
import { ref } from 'vue';

export function useSortPreference(context: keyof AppPageProps['sort_preferences']) {
    const page = usePage();

    const sortPreferences = ref(page.props.sort_preferences[context]);

    function updateSortPreferences(column: string, direction: 'asc' | 'desc') {
        sortPreferences.value = { column, direction };
        setCookie('sort_preferences', JSON.stringify({ ...page.props.sort_preferences, [context]: sortPreferences.value }), 365);
    }

    return {
        sortPreferences,
        updateSortPreferences,
    };
}
