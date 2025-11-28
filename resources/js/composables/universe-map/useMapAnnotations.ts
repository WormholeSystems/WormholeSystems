import { useStorage } from '@vueuse/core';
import { ref } from 'vue';

export type TAnnotationType = 'rect' | 'polygon';

export interface TAnnotation {
    id: string;
    type: TAnnotationType;
    // For rect
    x?: number;
    y?: number;
    width?: number;
    height?: number;
    // For polygon
    points?: { x: number; y: number }[];
    text: string;
    color: string;
}

export type TDrawMode = 'rect' | 'polygon' | 'eraser';

export interface UseMapAnnotationsOptions {
    storageKey?: string;
    defaultColor?: string;
    colors?: string[];
}

/**
 * Default annotation colors (hex values for canvas rendering)
 * Matches Tailwind: red-500, orange-500, yellow-500, green-500, blue-500, violet-500, pink-500
 */
const DEFAULT_COLORS = ['#ef4444', '#f97316', '#eab308', '#22c55e', '#3b82f6', '#8b5cf6', '#ec4899'];

/**
 * Annotation colors as CSS variables (for DOM elements)
 */
export const ANNOTATION_COLOR_VARS = [
    'var(--color-red-500)',
    'var(--color-orange-500)',
    'var(--color-yellow-500)',
    'var(--color-green-500)',
    'var(--color-blue-500)',
    'var(--color-violet-500)',
    'var(--color-pink-500)',
];

export function useMapAnnotations(options: UseMapAnnotationsOptions = {}) {
    const { storageKey = 'universe-map-annotations', defaultColor = '#3b82f6', colors = DEFAULT_COLORS } = options;

    // Use VueUse's useStorage for automatic localStorage sync
    const annotations = useStorage<TAnnotation[]>(storageKey, []);

    const isDrawingMode = ref(false);
    const drawMode = ref<TDrawMode>('rect');
    const isDrawing = ref(false);
    const drawStart = ref({ x: 0, y: 0 });
    const drawCurrent = ref({ x: 0, y: 0 });
    const polygonPoints = ref<{ x: number; y: number }[]>([]);
    const editingAnnotation = ref<TAnnotation | null>(null);
    const hoveredAnnotation = ref<TAnnotation | null>(null);
    const selectedColor = ref(defaultColor);

    /**
     * Check if a point is inside an annotation
     */
    function isPointInAnnotation(px: number, py: number, ann: TAnnotation): boolean {
        if (ann.type === 'rect' && ann.x !== undefined && ann.y !== undefined && ann.width !== undefined && ann.height !== undefined) {
            return px >= ann.x && px <= ann.x + ann.width && py >= ann.y && py <= ann.y + ann.height;
        } else if (ann.type === 'polygon' && ann.points && ann.points.length >= 3) {
            // Ray casting algorithm for polygon
            let inside = false;
            const points = ann.points;
            for (let i = 0, j = points.length - 1; i < points.length; j = i++) {
                const xi = points[i].x,
                    yi = points[i].y;
                const xj = points[j].x,
                    yj = points[j].y;
                if (yi > py !== yj > py && px < ((xj - xi) * (py - yi)) / (yj - yi) + xi) {
                    inside = !inside;
                }
            }
            return inside;
        }
        return false;
    }

    /**
     * Get annotation centroid for label positioning
     */
    function getAnnotationCenter(ann: TAnnotation): { x: number; y: number } {
        if (ann.type === 'rect' && ann.x !== undefined && ann.y !== undefined && ann.width !== undefined && ann.height !== undefined) {
            return { x: ann.x + ann.width / 2, y: ann.y };
        } else if (ann.type === 'polygon' && ann.points && ann.points.length >= 3) {
            let minX = Infinity,
                maxX = -Infinity,
                minY = Infinity;
            for (const p of ann.points) {
                if (p.x < minX) minX = p.x;
                if (p.x > maxX) maxX = p.x;
                if (p.y < minY) minY = p.y;
            }
            return { x: (minX + maxX) / 2, y: minY };
        }
        return { x: 0, y: 0 };
    }

    /**
     * Find annotation at a point
     */
    function findAnnotationAtPoint(px: number, py: number): TAnnotation | null {
        for (const ann of annotations.value) {
            if (isPointInAnnotation(px, py, ann)) {
                return ann;
            }
        }
        return null;
    }

    /**
     * Add a new annotation
     */
    function addAnnotation(ann: Omit<TAnnotation, 'id'>) {
        const newAnnotation: TAnnotation = {
            ...ann,
            id: Date.now().toString(),
        };
        annotations.value.push(newAnnotation);
        return newAnnotation;
    }

    /**
     * Delete an annotation by ID
     */
    function deleteAnnotation(id: string) {
        annotations.value = annotations.value.filter((a) => a.id !== id);
    }

    /**
     * Update an annotation
     */
    function updateAnnotation(id: string, updates: Partial<TAnnotation>) {
        const index = annotations.value.findIndex((a) => a.id === id);
        if (index !== -1) {
            annotations.value[index] = { ...annotations.value[index], ...updates };
        }
    }

    /**
     * Clear all annotations
     */
    function clearAnnotations() {
        annotations.value = [];
    }

    /**
     * Start drawing a rectangle
     */
    function startRect(x: number, y: number) {
        isDrawing.value = true;
        drawStart.value = { x, y };
        drawCurrent.value = { x, y };
    }

    /**
     * Add a point to the current polygon
     */
    function addPolygonPoint(x: number, y: number) {
        polygonPoints.value.push({ x, y });
        drawCurrent.value = { x, y };
    }

    /**
     * Finish drawing a rectangle
     */
    function finishRect(): TAnnotation | null {
        if (!isDrawing.value) return null;

        const x = Math.min(drawStart.value.x, drawCurrent.value.x);
        const y = Math.min(drawStart.value.y, drawCurrent.value.y);
        const width = Math.abs(drawCurrent.value.x - drawStart.value.x);
        const height = Math.abs(drawCurrent.value.y - drawStart.value.y);

        isDrawing.value = false;

        // Only create annotation if it has some size
        if (width > 10 && height > 10) {
            return addAnnotation({
                type: 'rect',
                x,
                y,
                width,
                height,
                text: '',
                color: selectedColor.value,
            });
        }

        return null;
    }

    /**
     * Finish drawing a polygon
     */
    function finishPolygon(): TAnnotation | null {
        if (polygonPoints.value.length < 3) {
            polygonPoints.value = [];
            return null;
        }

        const annotation = addAnnotation({
            type: 'polygon',
            points: [...polygonPoints.value],
            text: '',
            color: selectedColor.value,
        });

        polygonPoints.value = [];
        return annotation;
    }

    /**
     * Cancel current drawing
     */
    function cancelDrawing() {
        isDrawing.value = false;
        polygonPoints.value = [];
        drawStart.value = { x: 0, y: 0 };
        drawCurrent.value = { x: 0, y: 0 };
    }

    /**
     * Exit drawing mode entirely
     */
    function exitDrawingMode() {
        cancelDrawing();
        isDrawingMode.value = false;
        editingAnnotation.value = null;
        hoveredAnnotation.value = null;
    }

    return {
        annotations,
        isDrawingMode,
        drawMode,
        isDrawing,
        drawStart,
        drawCurrent,
        polygonPoints,
        editingAnnotation,
        hoveredAnnotation,
        selectedColor,
        colors,
        isPointInAnnotation,
        getAnnotationCenter,
        findAnnotationAtPoint,
        addAnnotation,
        deleteAnnotation,
        updateAnnotation,
        clearAnnotations,
        startRect,
        addPolygonPoint,
        finishRect,
        finishPolygon,
        cancelDrawing,
        exitDrawingMode,
    };
}
