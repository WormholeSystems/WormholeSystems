import { ref, watch } from 'vue';

const backgroundImageUrl = ref<string>('');

export function useMapBackground() {
    // Load from localStorage on initialization
    if (typeof window !== 'undefined') {
        const saved = localStorage.getItem('map-background-image');
        if (saved) {
            backgroundImageUrl.value = saved;
        }
    }

    // Save to localStorage whenever the value changes
    watch(backgroundImageUrl, (newUrl) => {
        if (typeof window !== 'undefined') {
            if (newUrl) {
                localStorage.setItem('map-background-image', newUrl);
            } else {
                localStorage.removeItem('map-background-image');
            }
        }
    });

    function setBackgroundImageUrl(url: string) {
        backgroundImageUrl.value = url;
    }

    function clearBackgroundImage() {
        backgroundImageUrl.value = '';
    }

    return {
        backgroundImageUrl,
        setBackgroundImageUrl,
        clearBackgroundImage,
    };
}
