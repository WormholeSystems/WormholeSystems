import { MaybeRefOrGetter, ref, toValue } from 'vue';

export interface PanZoomOptions {
    minScale?: number;
    maxScale?: number;
    initialScale?: number;
}

export function useCanvasPanZoom(container: MaybeRefOrGetter<HTMLElement | null>, options: PanZoomOptions = {}) {
    const { minScale = 0.02, maxScale = 5, initialScale = 0.5 } = options;

    const scale = ref(initialScale);
    const panOffset = ref({ x: 0, y: 0 });
    const isPanning = ref(false);
    const lastMousePosition = ref({ x: 0, y: 0 });

    // Animation state
    let animationFrameId: number | null = null;

    /**
     * Convert screen coordinates to world coordinates
     */
    function screenToWorld(screenX: number, screenY: number) {
        const containerEl = toValue(container);
        if (!containerEl) return { x: 0, y: 0 };

        const rect = containerEl.getBoundingClientRect();
        return {
            x: (screenX - rect.left - panOffset.value.x) / scale.value,
            y: (screenY - rect.top - panOffset.value.y) / scale.value,
        };
    }

    /**
     * Convert world coordinates to screen coordinates
     */
    function worldToScreen(worldX: number, worldY: number) {
        return {
            x: worldX * scale.value + panOffset.value.x,
            y: worldY * scale.value + panOffset.value.y,
        };
    }

    /**
     * Handle mouse wheel zoom (zooms towards cursor position)
     */
    function handleWheel(event: WheelEvent) {
        const containerEl = toValue(container);
        if (!containerEl) return;

        const rect = containerEl.getBoundingClientRect();
        const mouseX = event.clientX - rect.left;
        const mouseY = event.clientY - rect.top;

        const zoomFactor = event.deltaY > 0 ? 0.9 : 1.1;
        const newScale = Math.max(minScale, Math.min(maxScale, scale.value * zoomFactor));

        // Zoom towards mouse position
        const scaleRatio = newScale / scale.value;
        const newPanX = mouseX - (mouseX - panOffset.value.x) * scaleRatio;
        const newPanY = mouseY - (mouseY - panOffset.value.y) * scaleRatio;

        scale.value = newScale;
        panOffset.value = { x: newPanX, y: newPanY };
    }

    /**
     * Start panning
     */
    function startPan(event: MouseEvent) {
        isPanning.value = true;
        lastMousePosition.value = { x: event.clientX, y: event.clientY };
    }

    /**
     * Update pan position
     */
    function updatePan(event: MouseEvent) {
        if (!isPanning.value) return false;

        const deltaX = event.clientX - lastMousePosition.value.x;
        const deltaY = event.clientY - lastMousePosition.value.y;

        panOffset.value = {
            x: panOffset.value.x + deltaX,
            y: panOffset.value.y + deltaY,
        };

        lastMousePosition.value = { x: event.clientX, y: event.clientY };
        return true;
    }

    /**
     * Stop panning
     */
    function stopPan() {
        isPanning.value = false;
    }

    /**
     * Animate to a target scale and position
     */
    function animateTo(targetScale: number, targetPanX: number, targetPanY: number, duration = 400, onFrame?: () => void) {
        // Cancel any existing animation
        if (animationFrameId !== null) {
            cancelAnimationFrame(animationFrameId);
        }

        const startScale = scale.value;
        const startPanX = panOffset.value.x;
        const startPanY = panOffset.value.y;
        const startTime = performance.now();

        // Easing function (easeOutCubic)
        const ease = (t: number) => 1 - Math.pow(1 - t, 3);

        function animate(currentTime: number) {
            const elapsed = currentTime - startTime;
            const progress = Math.min(elapsed / duration, 1);
            const easedProgress = ease(progress);

            scale.value = startScale + (targetScale - startScale) * easedProgress;
            panOffset.value = {
                x: startPanX + (targetPanX - startPanX) * easedProgress,
                y: startPanY + (targetPanY - startPanY) * easedProgress,
            };

            onFrame?.();

            if (progress < 1) {
                animationFrameId = requestAnimationFrame(animate);
            } else {
                animationFrameId = null;
            }
        }

        animationFrameId = requestAnimationFrame(animate);
    }

    /**
     * Center the view on a specific world coordinate
     */
    function centerOn(worldX: number, worldY: number, containerWidth: number, containerHeight: number, targetScale?: number) {
        const newScale = targetScale ?? scale.value;
        const targetPanX = containerWidth / 2 - worldX * newScale;
        const targetPanY = containerHeight / 2 - worldY * newScale;

        return { scale: newScale, panX: targetPanX, panY: targetPanY };
    }

    /**
     * Fit bounds into view
     */
    function fitBounds(minX: number, minY: number, maxX: number, maxY: number, containerWidth: number, containerHeight: number, padding = 50) {
        const boundsWidth = maxX - minX;
        const boundsHeight = maxY - minY;

        if (boundsWidth <= 0 || boundsHeight <= 0) return null;

        const scaleX = (containerWidth - padding * 2) / boundsWidth;
        const scaleY = (containerHeight - padding * 2) / boundsHeight;
        const newScale = Math.max(minScale, Math.min(maxScale, Math.min(scaleX, scaleY)));

        const centerX = (minX + maxX) / 2;
        const centerY = (minY + maxY) / 2;

        const targetPanX = containerWidth / 2 - centerX * newScale;
        const targetPanY = containerHeight / 2 - centerY * newScale;

        return { scale: newScale, panX: targetPanX, panY: targetPanY };
    }

    /**
     * Set scale with bounds checking
     */
    function setScale(newScale: number) {
        scale.value = Math.max(minScale, Math.min(maxScale, newScale));
    }

    /**
     * Cleanup animation on unmount
     */
    function cleanup() {
        if (animationFrameId !== null) {
            cancelAnimationFrame(animationFrameId);
            animationFrameId = null;
        }
    }

    return {
        scale,
        panOffset,
        isPanning,
        screenToWorld,
        worldToScreen,
        handleWheel,
        startPan,
        updatePan,
        stopPan,
        animateTo,
        centerOn,
        fitBounds,
        setScale,
        cleanup,
    };
}
