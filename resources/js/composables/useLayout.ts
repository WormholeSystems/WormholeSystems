import { setCookie } from '@/lib/utils';
import { usePage } from '@inertiajs/vue3';
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

    function setLayout(newLayout: TLayout) {
        layout.value = newLayout;
        setCookie('layout', JSON.stringify(newLayout), 365);
    }

    return {
        layout,
        setLayout,
    };
}
