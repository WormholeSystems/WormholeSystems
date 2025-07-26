import { TSolarsystem } from '@/types/models';
import { readonly, ref } from 'vue';

const path = ref<TSolarsystem[] | null>(null);

export function usePath() {
    function setPath(newPath: TSolarsystem[] | null) {
        path.value = newPath;
    }

    return {
        path: readonly(path),
        setPath,
    };
}
