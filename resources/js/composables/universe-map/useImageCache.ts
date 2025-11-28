import { ref } from 'vue';

export function useImageCache(onImageLoaded?: () => void) {
    const cache = new Map<string, HTMLImageElement | null>();
    const pending = new Set<string>();
    const needsRedraw = ref(false);

    /**
     * Get an image from cache or start loading it
     * Returns the image if cached and loaded, null otherwise
     */
    function getImage(url: string): HTMLImageElement | null {
        if (cache.has(url)) {
            return cache.get(url) || null;
        }

        if (pending.has(url)) return null;

        // Start loading
        pending.add(url);
        cache.set(url, null);

        const img = new Image();
        img.crossOrigin = 'anonymous';
        img.onload = () => {
            cache.set(url, img);
            pending.delete(url);
            needsRedraw.value = true;
            onImageLoaded?.();
        };
        img.onerror = () => {
            cache.set(url, null);
            pending.delete(url);
        };
        img.src = url;

        return null;
    }

    /**
     * Preload multiple images
     */
    function preload(urls: string[]) {
        for (const url of urls) {
            getImage(url);
        }
    }

    /**
     * Check if an image is loaded
     */
    function isLoaded(url: string): boolean {
        return cache.has(url) && cache.get(url) !== null;
    }

    /**
     * Check if an image is currently loading
     */
    function isLoading(url: string): boolean {
        return pending.has(url);
    }

    /**
     * Clear the cache
     */
    function clear() {
        cache.clear();
        pending.clear();
    }

    /**
     * Get cache stats
     */
    function getStats() {
        let loaded = 0;
        let failed = 0;
        for (const [, img] of cache) {
            if (img) loaded++;
            else failed++;
        }
        return {
            loaded,
            failed,
            pending: pending.size,
            total: cache.size,
        };
    }

    return {
        getImage,
        preload,
        isLoaded,
        isLoading,
        clear,
        getStats,
        needsRedraw,
    };
}
