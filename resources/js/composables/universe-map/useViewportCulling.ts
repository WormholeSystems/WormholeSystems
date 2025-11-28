import { computed, MaybeRefOrGetter, toValue } from 'vue';

export interface ViewportBounds {
    left: number;
    right: number;
    top: number;
    bottom: number;
}

export interface UseViewportCullingOptions<T> {
    items: MaybeRefOrGetter<T[]>;
    getPosition: (item: T) => { x: number; y: number };
    scale: MaybeRefOrGetter<number>;
    panOffset: MaybeRefOrGetter<{ x: number; y: number }>;
    containerWidth: MaybeRefOrGetter<number>;
    containerHeight: MaybeRefOrGetter<number>;
    padding?: number;
}

export function useViewportCulling<T>(options: UseViewportCullingOptions<T>) {
    const { items, getPosition, scale, panOffset, containerWidth, containerHeight, padding = 100 } = options;

    /**
     * Get current viewport bounds in world coordinates
     */
    const viewportBounds = computed<ViewportBounds>(() => {
        const s = toValue(scale);
        const pan = toValue(panOffset);
        const width = toValue(containerWidth);
        const height = toValue(containerHeight);

        if (width === 0 || height === 0) {
            return { left: 0, right: 0, top: 0, bottom: 0 };
        }

        return {
            left: -pan.x / s,
            right: (-pan.x + width) / s,
            top: -pan.y / s,
            bottom: (-pan.y + height) / s,
        };
    });

    /**
     * Get items visible in the current viewport
     */
    const visibleItems = computed(() => {
        const allItems = toValue(items);
        const bounds = viewportBounds.value;

        if (bounds.left === bounds.right) return [];

        return allItems.filter((item) => {
            const pos = getPosition(item);
            return (
                pos.x >= bounds.left - padding && pos.x <= bounds.right + padding && pos.y >= bounds.top - padding && pos.y <= bounds.bottom + padding
            );
        });
    });

    /**
     * Check if a point is visible in the viewport
     */
    function isPointVisible(x: number, y: number): boolean {
        const bounds = viewportBounds.value;
        return x >= bounds.left - padding && x <= bounds.right + padding && y >= bounds.top - padding && y <= bounds.bottom + padding;
    }

    /**
     * Check if a rectangle intersects the viewport
     */
    function isRectVisible(x: number, y: number, width: number, height: number): boolean {
        const bounds = viewportBounds.value;
        return !(x + width < bounds.left - padding || x > bounds.right + padding || y + height < bounds.top - padding || y > bounds.bottom + padding);
    }

    /**
     * Check if a line segment intersects the viewport (for connection culling)
     */
    function isLineVisible(x1: number, y1: number, x2: number, y2: number): boolean {
        const bounds = viewportBounds.value;
        const vLeft = bounds.left - padding;
        const vRight = bounds.right + padding;
        const vTop = bounds.top - padding;
        const vBottom = bounds.bottom + padding;

        // Quick check: if both endpoints are visible
        const p1Inside = x1 >= vLeft && x1 <= vRight && y1 >= vTop && y1 <= vBottom;
        const p2Inside = x2 >= vLeft && x2 <= vRight && y2 >= vTop && y2 <= vBottom;
        if (p1Inside || p2Inside) return true;

        // Check if line intersects any viewport edge
        return (
            lineIntersectsLine(x1, y1, x2, y2, vLeft, vTop, vRight, vTop) || // Top
            lineIntersectsLine(x1, y1, x2, y2, vLeft, vBottom, vRight, vBottom) || // Bottom
            lineIntersectsLine(x1, y1, x2, y2, vLeft, vTop, vLeft, vBottom) || // Left
            lineIntersectsLine(x1, y1, x2, y2, vRight, vTop, vRight, vBottom) // Right
        );
    }

    return {
        viewportBounds,
        visibleItems,
        isPointVisible,
        isRectVisible,
        isLineVisible,
    };
}

/**
 * Check if two line segments intersect
 */
function lineIntersectsLine(x1: number, y1: number, x2: number, y2: number, x3: number, y3: number, x4: number, y4: number): boolean {
    const denom = (y4 - y3) * (x2 - x1) - (x4 - x3) * (y2 - y1);
    if (Math.abs(denom) < 0.0001) return false;

    const ua = ((x4 - x3) * (y1 - y3) - (y4 - y3) * (x1 - x3)) / denom;
    const ub = ((x2 - x1) * (y1 - y3) - (y2 - y1) * (x1 - x3)) / denom;

    return ua >= 0 && ua <= 1 && ub >= 0 && ub <= 1;
}
