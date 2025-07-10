import { AppPageProps } from '@/types';
import { router, usePage } from '@inertiajs/vue3';
import { useDebounceFn } from '@vueuse/core';
import { ref, watch } from 'vue';

export function useSearch(name = 'search', only: string[] = []) {
    const page = usePage<
        AppPageProps<{
            [name: string]: string;
        }>
    >();

    const search = ref(page.props[name] || '');

    const debounced = useDebounceFn(refreshSearch, 150, {
        maxWait: 500,
    });

    watch(search, debounced);

    function refreshSearch() {
        router.reload({
            data: {
                [name]: search.value,
            },
            only: only,
        });
    }

    return search;
}
