import { setCookie } from '@/lib/utils';
import { usePage } from '@inertiajs/vue3';
import { ref } from 'vue';

export type TLayout = {
    map_height: number;
};

export function useLayout() {
    const page = usePage();
    const layout = ref<TLayout>(page.props.layout);

    function setLayout(newLayout: TLayout) {
        layout.value = newLayout;
        setCookie('layout', JSON.stringify(newLayout), 365);
    }

    return {
        layout,
        setLayout,
    };
}
