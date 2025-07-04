import { onMounted, ref } from 'vue';

export function useOS() {
    const os = ref<'mac' | 'other'>('other');

    onMounted(() => {
        const platform = navigator.platform.toLowerCase();
        if (platform.includes('mac')) {
            os.value = 'mac';
        }
    });

    return os;
}
