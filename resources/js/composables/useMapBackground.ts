import { ref, watch } from 'vue';
import { toast } from 'vue-sonner';

/**
 * How the background image is laid out:
 * - `grid`: stretched across the whole map grid; pans and zooms with the systems.
 * - `viewport`: sized to the visible map panel and kept static while panning/zooming.
 */
export type TMapBackgroundMode = 'grid' | 'viewport';

const URL_STORAGE_KEY = 'map-background-image';
const MODE_STORAGE_KEY = 'map-background-mode';

const backgroundImageUrl = ref<string>('');
const backgroundMode = ref<TMapBackgroundMode>('grid');

let initialized = false;

export function useMapBackground() {
    if (typeof window !== 'undefined' && !initialized) {
        initialized = true;

        const savedUrl = localStorage.getItem(URL_STORAGE_KEY);
        if (savedUrl) {
            backgroundImageUrl.value = savedUrl;
        }

        const savedMode = localStorage.getItem(MODE_STORAGE_KEY);
        if (savedMode === 'grid' || savedMode === 'viewport') {
            backgroundMode.value = savedMode;
        }

        watch(backgroundImageUrl, (newUrl) => {
            if (!newUrl) {
                localStorage.removeItem(URL_STORAGE_KEY);
                return;
            }

            try {
                localStorage.setItem(URL_STORAGE_KEY, newUrl);
            } catch {
                // Uploaded data URLs can exceed the localStorage quota. Keep the
                // image for this session, but let the user know it won't persist.
                toast.warning('Background image is too large to save', {
                    description: 'It will be shown until you reload. Try a smaller image or use a URL instead.',
                });
            }
        });

        watch(backgroundMode, (mode) => {
            localStorage.setItem(MODE_STORAGE_KEY, mode);
        });
    }

    function setBackgroundImageUrl(url: string) {
        backgroundImageUrl.value = url;
    }

    function clearBackgroundImage() {
        backgroundImageUrl.value = '';
    }

    function setBackgroundMode(mode: TMapBackgroundMode) {
        backgroundMode.value = mode;
    }

    return {
        backgroundImageUrl,
        backgroundMode,
        setBackgroundImageUrl,
        clearBackgroundImage,
        setBackgroundMode,
    };
}
