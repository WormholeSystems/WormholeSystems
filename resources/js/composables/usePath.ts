import { readonly, ref } from 'vue';

const path = ref<number[] | null>(null);

export function usePath() {
    function setPath(newPath: number[] | null) {
        path.value = newPath;
    }

    return {
        path: readonly(path),
        setPath,
    };
}
