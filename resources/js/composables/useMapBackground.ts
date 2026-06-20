import { useMapUserSettings } from '@/composables/useMapUserSettings';
import { computed } from 'vue';

/**
 * How the background image is laid out:
 * - `grid`: stretched across the whole map grid; pans and zooms with the systems.
 * - `viewport`: sized to the visible map panel and kept static while panning/zooming.
 */
export type TMapBackgroundMode = 'grid' | 'viewport';

/**
 * The map background is persisted in the user's map settings, so it follows them
 * across devices. Authenticated users can upload an image; guests cannot persist
 * one and simply see no background.
 */
export function useMapBackground() {
    const settings = useMapUserSettings();

    const canUpload = computed(() => Boolean(settings.value.user_id));
    const backgroundImageUrl = computed<string>(() => settings.value.background_image_url ?? '');
    const backgroundMode = computed<TMapBackgroundMode>(() => settings.value.background_image_mode ?? 'grid');

    return {
        backgroundImageUrl,
        backgroundMode,
        canUpload,
    };
}
