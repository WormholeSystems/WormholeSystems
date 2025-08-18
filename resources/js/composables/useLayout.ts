import { setCookie } from '@/lib/utils';
import { router, usePage } from '@inertiajs/vue3';
import { useDebounceFn } from '@vueuse/core';
import { ref, watch } from 'vue';

export type TLayout = {
    map_height: number;
    scale: number;
};

const layout = ref<TLayout>({
    map_height: 1000,
    scale: 1,
});

export function useLayout() {
    const page = usePage();

    watch(
        () => page.props.layout,
        (newLayout) => {
            if (newLayout) {
                layout.value = newLayout;
            }
        },
        { immediate: true },
    );

    function syncLayout() {
        router.reload({
            only: ['layout'],
        });
    }

    const debouncedSyncLayout = useDebounceFn(syncLayout, 1000);

    function setLayout(newLayout: TLayout) {
        layout.value = newLayout;
        setCookie('layout', JSON.stringify(newLayout), 365);
        debouncedSyncLayout();
    }

    return {
        layout,
        setLayout,
    };
}
