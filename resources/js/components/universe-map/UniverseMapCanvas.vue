<script setup lang="ts">
import { TUniverseBounds, TUniverseConnection, TUniverseSolarsystem } from '@/types/universe-map';
import { useElementSize, useMagicKeys, useThrottleFn, whenever } from '@vueuse/core';
import { Delaunay } from 'd3-delaunay';
import { Eraser, PenTool, Square } from 'lucide-vue-next';
import { computed, onMounted, onUnmounted, ref, shallowRef, useTemplateRef, watch } from 'vue';

const props = defineProps<{
    solarsystems: TUniverseSolarsystem[];
    connections: TUniverseConnection[];
    bounds: TUniverseBounds;
}>();

const scale = defineModel<number>('scale', { default: 0.5 });
const panOffset = defineModel<{ x: number; y: number }>('panOffset', { default: () => ({ x: 0, y: 0 }) });

const emit = defineEmits<{
    'system-click': [system: TUniverseSolarsystem];
    'system-contextmenu': [system: TUniverseSolarsystem, position: { x: number; y: number }];
}>();

// Image cache for sovereignty logos
const imageCache = new Map<string, HTMLImageElement | null>();
const pendingImages = new Set<string>();
const needsRedraw = ref(false);

function getSovLogoUrl(system: TUniverseSolarsystem): string | null {
    if (system.sovereignty?.alliance) {
        return `https://images.evetech.net/alliances/${system.sovereignty.alliance.id}/logo?size=32`;
    }
    if (system.sovereignty?.corporation) {
        return `https://images.evetech.net/corporations/${system.sovereignty.corporation.id}/logo?size=32`;
    }
    if (system.sovereignty?.faction) {
        return `https://images.evetech.net/corporations/${system.sovereignty.faction.id}/logo?size=32`;
    }
    return null;
}

function loadImage(url: string): HTMLImageElement | null {
    if (imageCache.has(url)) {
        return imageCache.get(url) || null;
    }

    if (pendingImages.has(url)) return null;

    // Start loading
    pendingImages.add(url);
    imageCache.set(url, null);
    const img = new Image();
    img.crossOrigin = 'anonymous';
    img.onload = () => {
        imageCache.set(url, img);
        pendingImages.delete(url);
        needsRedraw.value = true; // Trigger redraw when image loads
    };
    img.onerror = () => {
        imageCache.set(url, null);
        pendingImages.delete(url);
    };
    img.src = url;

    return null;
}

const containerRef = useTemplateRef<HTMLDivElement>('container');
const canvasRef = useTemplateRef<HTMLCanvasElement>('canvas');
const intelInputRef = useTemplateRef<HTMLInputElement>('intelInputRef');
const { width: containerWidth, height: containerHeight } = useElementSize(containerRef);

// Track hovered system for tooltip
const hoveredSystem = shallowRef<TUniverseSolarsystem | null>(null);
const tooltipPosition = ref({ x: 0, y: 0 });

// Highlight state for focused systems
const highlightedRegionId = ref<number | null>(null);
const highlightedConstellationId = ref<number | null>(null);
const highlightedSystemId = ref<number | null>(null);

// Panning state
const isPanning = ref(false);
const lastMousePosition = ref({ x: 0, y: 0 });
const mouseDownPosition = ref<{ x: number; y: number; worldX: number; worldY: number } | null>(null);

// Animation state
let animationFrameId: number | null = null;

function animateTo(targetScale: number, targetPanX: number, targetPanY: number, duration = 400) {
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

        if (progress < 1) {
            animationFrameId = requestAnimationFrame(animate);
        } else {
            animationFrameId = null;
        }

        requestDraw();
    }

    animationFrameId = requestAnimationFrame(animate);
}

// Intel/Annotation system
type TAnnotation = {
    id: string;
    type: 'rect' | 'polygon';
    // For rect
    x?: number;
    y?: number;
    width?: number;
    height?: number;
    // For polygon
    points?: { x: number; y: number }[];
    text: string;
    color: string;
};

type TDrawMode = 'rect' | 'polygon' | 'eraser';

const isDrawingMode = ref(false);
const drawMode = ref<TDrawMode>('rect');
const isDrawing = ref(false);
const drawStart = ref({ x: 0, y: 0 });
const drawCurrent = ref({ x: 0, y: 0 });
const polygonPoints = ref<{ x: number; y: number }[]>([]);
const editingAnnotation = ref<TAnnotation | null>(null);
const hoveredAnnotation = ref<TAnnotation | null>(null);
const annotationColors = ['#ef4444', '#f97316', '#eab308', '#22c55e', '#3b82f6', '#8b5cf6', '#ec4899'];
const selectedColor = ref(annotationColors[4]); // Default blue

// Load annotations from localStorage
const STORAGE_KEY = 'universe-map-annotations';
const annotations = ref<TAnnotation[]>([]);

function loadAnnotations() {
    try {
        const stored = localStorage.getItem(STORAGE_KEY);
        if (stored) {
            annotations.value = JSON.parse(stored);
        }
    } catch (e) {
        console.warn('Failed to load annotations from localStorage', e);
    }
}

function saveAnnotations() {
    try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(annotations.value));
    } catch (e) {
        console.warn('Failed to save annotations to localStorage', e);
    }
}

// Watch for annotation changes and save
watch(annotations, saveAnnotations, { deep: true });

// Check if point is inside annotation
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

// Get annotation centroid for label positioning
function getAnnotationCenter(ann: TAnnotation): { x: number; y: number } {
    if (ann.type === 'rect' && ann.x !== undefined && ann.y !== undefined && ann.width !== undefined && ann.height !== undefined) {
        return { x: ann.x + ann.width / 2, y: ann.y };
    } else if (ann.type === 'polygon' && ann.points && ann.points.length >= 3) {
        // Find top-center of bounding box
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

// Canvas DPR for sharp rendering
const dpr = ref(typeof window !== 'undefined' ? window.devicePixelRatio || 1 : 1);

// Calculate the world size (with padding)
const worldPadding = 200; // Padding around the map
const worldWidth = computed(() => {
    const width = props.bounds.maxX - props.bounds.minX;
    return isFinite(width) && width > 0 ? width + worldPadding * 2 : 10000;
});
const worldHeight = computed(() => {
    const height = props.bounds.maxY - props.bounds.minY;
    return isFinite(height) && height > 0 ? height + worldPadding * 2 : 10000;
});

// Transform world coordinates to canvas coordinates (Y axis inverted)
function worldToCanvas(worldX: number, worldY: number): { x: number; y: number } {
    const minX = isFinite(props.bounds.minX) ? props.bounds.minX : 0;
    const minY = isFinite(props.bounds.minY) ? props.bounds.minY : 0;
    const maxY = isFinite(props.bounds.maxY) ? props.bounds.maxY : 0;
    const height = maxY - minY;
    return {
        x: worldX - minX + worldPadding,
        y: height - (worldY - minY) + worldPadding, // Invert Y axis
    };
}

// Pre-calculate canvas positions for all systems
const systemPositions = computed(() => {
    return props.solarsystems.map((system) => {
        const pos = worldToCanvas(system.position.x, system.position.y);
        return {
            system,
            x: pos.x,
            y: pos.y,
        };
    });
});

// Create a lookup map for system positions by ID
const systemPositionMap = computed(() => {
    const map = new Map<number, { x: number; y: number }>();
    for (const { system, x, y } of systemPositions.value) {
        map.set(system.id, { x, y });
    }
    return map;
});

// Create a map of connections for each system (for drawing direction arrows)
const systemConnectionsMap = computed(() => {
    const map = new Map<number, { x: number; y: number; regional: boolean }[]>();
    const posMap = systemPositionMap.value;

    for (const conn of props.connections) {
        const fromPos = posMap.get(conn.from);
        const toPos = posMap.get(conn.to);

        if (!fromPos || !toPos) continue;

        // Add connection direction from "from" system
        if (!map.has(conn.from)) map.set(conn.from, []);
        map.get(conn.from)!.push({ x: toPos.x, y: toPos.y, regional: conn.regional });

        // Add connection direction from "to" system (reverse)
        if (!map.has(conn.to)) map.set(conn.to, []);
        map.get(conn.to)!.push({ x: fromPos.x, y: fromPos.y, regional: conn.regional });
    }

    return map;
});

// Tailwind CSS color palette for regions
const regionPalette = [
    '239, 68, 68', // red-500
    '249, 115, 22', // orange-500
    '234, 179, 8', // yellow-500
    '132, 204, 22', // lime-500
    '34, 197, 94', // green-500
    '16, 185, 129', // emerald-500
    '20, 184, 166', // teal-500
    '6, 182, 212', // cyan-500
    '14, 165, 233', // sky-500
    '59, 130, 246', // blue-500
    '99, 102, 241', // indigo-500
    '139, 92, 246', // violet-500
    '168, 85, 247', // purple-500
    '217, 70, 239', // fuchsia-500
    '236, 72, 153', // pink-500
    '244, 63, 94', // rose-500
];

// Compute region boundary lines using d3-delaunay Voronoi
const regionData = computed(() => {
    const positions = systemPositions.value;
    if (positions.length < 3) return null;

    // Build region metadata
    const regionMeta = new Map<number, { name: string; color: string }>();
    for (const { system } of positions) {
        if (!regionMeta.has(system.region_id)) {
            const colorIndex = system.region_id % regionPalette.length;
            regionMeta.set(system.region_id, {
                name: system.region.name,
                color: regionPalette[colorIndex],
            });
        }
    }

    // Calculate bounds for Voronoi
    let minX = Infinity,
        maxX = -Infinity,
        minY = Infinity,
        maxY = -Infinity;
    for (const { x, y } of positions) {
        if (x < minX) minX = x;
        if (x > maxX) maxX = x;
        if (y < minY) minY = y;
        if (y > maxY) maxY = y;
    }
    const padding = 300;
    const bounds: [number, number, number, number] = [minX - padding, minY - padding, maxX + padding, maxY + padding];

    // Create Delaunay triangulation and Voronoi diagram
    const points = positions.map(({ x, y }) => [x, y] as [number, number]);
    const delaunay = Delaunay.from(points);
    const voronoi = delaunay.voronoi(bounds);

    // Find boundary edges between different regions
    const boundaryEdges: { x1: number; y1: number; x2: number; y2: number }[] = [];

    // Iterate over all Delaunay edges
    for (let i = 0; i < positions.length; i++) {
        const regionI = positions[i].system.region_id;

        // Get neighbors of point i
        for (const j of delaunay.neighbors(i)) {
            if (j <= i) continue; // Avoid duplicate edges

            const regionJ = positions[j].system.region_id;

            // If different regions, this is a boundary edge
            if (regionI !== regionJ) {
                // Get the Voronoi edge between cells i and j
                const cellI = voronoi.cellPolygon(i);
                const cellJ = voronoi.cellPolygon(j);
                if (!cellI || !cellJ) continue;

                // Find shared edge between the two cells
                for (let a = 0; a < cellI.length; a++) {
                    const p1 = cellI[a];
                    const p2 = cellI[(a + 1) % cellI.length];

                    // Check if this edge is shared with cellJ
                    for (let b = 0; b < cellJ.length; b++) {
                        const q1 = cellJ[b];
                        const q2 = cellJ[(b + 1) % cellJ.length];

                        // Check if edges match (in either direction)
                        if (
                            (Math.abs(p1[0] - q2[0]) < 0.1 &&
                                Math.abs(p1[1] - q2[1]) < 0.1 &&
                                Math.abs(p2[0] - q1[0]) < 0.1 &&
                                Math.abs(p2[1] - q1[1]) < 0.1) ||
                            (Math.abs(p1[0] - q1[0]) < 0.1 &&
                                Math.abs(p1[1] - q1[1]) < 0.1 &&
                                Math.abs(p2[0] - q2[0]) < 0.1 &&
                                Math.abs(p2[1] - q2[1]) < 0.1)
                        ) {
                            boundaryEdges.push({ x1: p1[0], y1: p1[1], x2: p2[0], y2: p2[1] });
                            break;
                        }
                    }
                }
            }
        }
    }

    // Calculate region centroids for labels
    const regionCentroids = new Map<number, { x: number; y: number; count: number }>();
    for (const pos of positions) {
        const regionId = pos.system.region_id;
        if (!regionCentroids.has(regionId)) {
            regionCentroids.set(regionId, { x: 0, y: 0, count: 0 });
        }
        const c = regionCentroids.get(regionId)!;
        c.x += pos.x;
        c.y += pos.y;
        c.count++;
    }

    const labels: { name: string; color: string; x: number; y: number }[] = [];
    for (const [regionId, c] of regionCentroids) {
        const meta = regionMeta.get(regionId);
        if (!meta || c.count === 0) continue;
        labels.push({
            name: meta.name,
            color: meta.color,
            x: c.x / c.count,
            y: c.y / c.count,
        });
    }

    return { boundaryEdges, labels };
});

// Get systems visible in viewport (culling)
const visibleSystems = computed(() => {
    if (containerWidth.value === 0 || containerHeight.value === 0) return [];

    const viewportLeft = -panOffset.value.x / scale.value;
    const viewportTop = -panOffset.value.y / scale.value;
    const viewportRight = viewportLeft + containerWidth.value / scale.value;
    const viewportBottom = viewportTop + containerHeight.value / scale.value;

    // Add padding to avoid popping
    const padding = 100;

    return systemPositions.value.filter(({ x, y }) => {
        return x >= viewportLeft - padding && x <= viewportRight + padding && y >= viewportTop - padding && y <= viewportBottom + padding;
    });
});

// Security color helper
function getSecurityColor(system: TUniverseSolarsystem): string {
    if (system.class) {
        const colors: Record<number, string> = {
            1: '#67e8f9',
            2: '#3b82f6',
            3: '#d8b4fe',
            4: '#8b5cf6',
            5: '#fb923c',
            6: '#ef4444',
        };
        return colors[system.class] ?? '#a3a3a3';
    }
    if (system.security >= 0.5) return '#22c55e';
    if (system.security >= 0.1) return '#f97316';
    return '#ef4444';
}

// Draw the canvas
function draw() {
    const canvas = canvasRef.value;
    const ctx = canvas?.getContext('2d');
    if (!canvas || !ctx) return;

    // Set canvas size with DPR for sharp rendering
    const displayWidth = containerWidth.value;
    const displayHeight = containerHeight.value;
    if (displayWidth === 0 || displayHeight === 0) return;

    canvas.width = displayWidth * dpr.value;
    canvas.height = displayHeight * dpr.value;
    canvas.style.width = displayWidth + 'px';
    canvas.style.height = displayHeight + 'px';
    ctx.scale(dpr.value, dpr.value);

    // Clear canvas with dark background
    ctx.fillStyle = '#0a0a0a';
    ctx.fillRect(0, 0, displayWidth, displayHeight);

    // Draw stars background
    drawStars(ctx, displayWidth, displayHeight);

    // Apply transform
    ctx.save();
    ctx.translate(panOffset.value.x, panOffset.value.y);
    ctx.scale(scale.value, scale.value);

    // Determine LOD based on scale
    const lod: 'micro' | 'minimal' | 'simple' | 'detailed' =
        scale.value < 0.05 ? 'micro' : scale.value < 0.15 ? 'minimal' : scale.value < 0.4 ? 'simple' : 'detailed';

    // Draw region outlines first (background layer)
    drawRegionOutlines(ctx);

    // Draw connections (so they appear behind systems)
    drawConnections(ctx, lod);

    // Draw systems
    const visible = visibleSystems.value;
    for (const { system, x, y } of visible) {
        drawSystem(ctx, system, x, y, lod);
    }

    // Draw region labels
    drawRegionLabels(ctx, lod);

    // Draw annotations (intel markers)
    drawAnnotations(ctx);

    ctx.restore();

    // Draw stats overlay
    ctx.fillStyle = '#525252'; // neutral-600
    ctx.font = '11px system-ui, sans-serif';
    ctx.fillText(
        `${visible.length} / ${props.solarsystems.length} systems | ${props.connections.length} connections | Zoom: ${Math.round(scale.value * 100)}%`,
        10,
        displayHeight - 10,
    );
}

// Draw simple star background
function drawStars(ctx: CanvasRenderingContext2D, width: number, height: number) {
    ctx.fillStyle = 'rgba(163, 163, 163, 0.08)'; // neutral-400
    const seed = 12345;
    for (let i = 0; i < 150; i++) {
        const x = ((seed * (i + 1) * 9301 + 49297) % 233280) / 233280;
        const y = ((seed * (i + 1) * 49297 + 9301) % 233280) / 233280;
        const size = ((seed * (i + 1) * 7919) % 10) / 10 + 0.3;
        ctx.beginPath();
        ctx.arc(x * width, y * height, size, 0, Math.PI * 2);
        ctx.fill();
    }
}

// Draw dotted region boundary lines
function drawRegionOutlines(ctx: CanvasRenderingContext2D) {
    const data = regionData.value;
    if (!data) return;

    const { boundaryEdges } = data;

    // Draw boundary edges as dotted lines (neutral styling, constant screen width)
    const screenLineWidth = 2; // Constant 2px on screen
    const worldLineWidth = screenLineWidth / scale.value;
    const dashScale = 1 / scale.value;
    ctx.setLineDash([8 * dashScale, 6 * dashScale]);
    ctx.strokeStyle = 'rgba(115, 115, 115, 0.4)'; // neutral-500
    ctx.lineWidth = worldLineWidth;

    ctx.beginPath();
    for (const edge of boundaryEdges) {
        ctx.moveTo(edge.x1, edge.y1);
        ctx.lineTo(edge.x2, edge.y2);
    }
    ctx.stroke();
    ctx.setLineDash([]);
}

// Draw region labels (called last to overlay everything)
function drawRegionLabels(ctx: CanvasRenderingContext2D, lod: 'micro' | 'minimal' | 'simple' | 'detailed') {
    if (lod === 'detailed' || scale.value <= 0.02) return;

    const data = regionData.value;
    if (!data) return;

    for (const label of data.labels) {
        ctx.fillStyle = '#ffffff';
        const fontSize = lod === 'micro' ? 220 : lod === 'minimal' ? 160 : 100;
        ctx.font = `600 ${fontSize}px system-ui, sans-serif`;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(label.name, label.x, label.y);
    }
}

// Check if a line segment intersects a rectangle
function lineIntersectsRect(
    x1: number,
    y1: number,
    x2: number,
    y2: number,
    rectLeft: number,
    rectTop: number,
    rectRight: number,
    rectBottom: number,
): boolean {
    // Check if either endpoint is inside the rect
    if (
        (x1 >= rectLeft && x1 <= rectRight && y1 >= rectTop && y1 <= rectBottom) ||
        (x2 >= rectLeft && x2 <= rectRight && y2 >= rectTop && y2 <= rectBottom)
    ) {
        return true;
    }

    // Check if line intersects any of the 4 edges of the rectangle
    return (
        lineSegmentsIntersect(x1, y1, x2, y2, rectLeft, rectTop, rectRight, rectTop) || // top
        lineSegmentsIntersect(x1, y1, x2, y2, rectLeft, rectBottom, rectRight, rectBottom) || // bottom
        lineSegmentsIntersect(x1, y1, x2, y2, rectLeft, rectTop, rectLeft, rectBottom) || // left
        lineSegmentsIntersect(x1, y1, x2, y2, rectRight, rectTop, rectRight, rectBottom) // right
    );
}

// Check if two line segments intersect
function lineSegmentsIntersect(x1: number, y1: number, x2: number, y2: number, x3: number, y3: number, x4: number, y4: number): boolean {
    const d = (x2 - x1) * (y4 - y3) - (y2 - y1) * (x4 - x3);
    if (d === 0) return false;

    const t = ((x3 - x1) * (y4 - y3) - (y3 - y1) * (x4 - x3)) / d;
    const u = -((x2 - x1) * (y3 - y1) - (y2 - y1) * (x3 - x1)) / d;

    return t >= 0 && t <= 1 && u >= 0 && u <= 1;
}

// Draw stargate connections
function drawConnections(ctx: CanvasRenderingContext2D, lod: 'micro' | 'minimal' | 'simple' | 'detailed') {
    const posMap = systemPositionMap.value;

    // Skip connections at very low zoom to improve performance
    if ((lod === 'micro' || lod === 'minimal') && props.connections.length > 5000) {
        return;
    }

    // Calculate viewport bounds in world coordinates
    const viewLeft = -panOffset.value.x / scale.value;
    const viewTop = -panOffset.value.y / scale.value;
    const viewRight = viewLeft + containerWidth.value / scale.value;
    const viewBottom = viewTop + containerHeight.value / scale.value;

    // Separate regular and regional connections
    const regularConns: { from: { x: number; y: number }; to: { x: number; y: number } }[] = [];
    const regionalConns: { from: { x: number; y: number }; to: { x: number; y: number } }[] = [];

    for (const conn of props.connections) {
        const from = posMap.get(conn.from);
        const to = posMap.get(conn.to);

        if (!from || !to) continue;

        // Check if line intersects viewport
        if (!lineIntersectsRect(from.x, from.y, to.x, to.y, viewLeft, viewTop, viewRight, viewBottom)) {
            continue;
        }

        if (conn.regional) {
            regionalConns.push({ from, to });
        } else {
            regularConns.push({ from, to });
        }
    }

    // Set common style for all connections (neutral styling)
    ctx.strokeStyle = lod === 'micro' ? 'rgba(82, 82, 82, 0.3)' : 'rgba(82, 82, 82, 0.5)'; // neutral-600
    ctx.lineWidth = lod === 'micro' ? 0.3 : lod === 'minimal' ? 0.4 : lod === 'simple' ? 0.5 : 0.6;

    // Draw regular connections (solid)
    ctx.setLineDash([]);
    ctx.beginPath();
    for (const { from, to } of regularConns) {
        ctx.moveTo(from.x, from.y);
        ctx.lineTo(to.x, to.y);
    }
    ctx.stroke();

    // Draw regional connections (dashed)
    if (regionalConns.length > 0) {
        ctx.setLineDash(lod === 'micro' ? [1, 1] : lod === 'minimal' ? [2, 2] : lod === 'simple' ? [4, 3] : [6, 4]);
        ctx.beginPath();
        for (const { from, to } of regionalConns) {
            ctx.moveTo(from.x, from.y);
            ctx.lineTo(to.x, to.y);
        }
        ctx.stroke();
        ctx.setLineDash([]);
    }
}

// Get security status label
function getSecurityLabel(system: TUniverseSolarsystem): string {
    if (system.class) return `C${system.class}`;
    if (system.security >= 0.5) return system.security.toFixed(1);
    if (system.security >= 0.1) return system.security.toFixed(1);
    return system.security.toFixed(1);
}

// Get sovereignty info
function getSovInfo(system: TUniverseSolarsystem): { name: string; type: 'alliance' | 'corp' | 'faction' } | null {
    if (system.sovereignty?.alliance) return { name: system.sovereignty.alliance.name, type: 'alliance' };
    if (system.sovereignty?.corporation) return { name: system.sovereignty.corporation.name, type: 'corp' };
    if (system.sovereignty?.faction) return { name: system.sovereignty.faction.name, type: 'faction' };
    return null;
}

// Check if a system is highlighted
function isSystemHighlighted(system: TUniverseSolarsystem): boolean {
    if (highlightedSystemId.value === system.id) return true;
    if (highlightedConstellationId.value === system.constellation_id) return true;
    if (highlightedRegionId.value === system.region_id) return true;
    return false;
}

// Draw a single system
function drawSystem(
    ctx: CanvasRenderingContext2D,
    system: TUniverseSolarsystem,
    x: number,
    y: number,
    lod: 'micro' | 'minimal' | 'simple' | 'detailed',
) {
    const isHovered = hoveredSystem.value?.id === system.id;
    const isHighlighted = isSystemHighlighted(system);
    const secColor = getSecurityColor(system);

    // Draw highlight ring first (behind the system)
    if (isHighlighted) {
        ctx.strokeStyle = '#facc15'; // yellow-400
        ctx.lineWidth = 2 / scale.value;
        ctx.globalAlpha = 0.8;
        ctx.beginPath();

        if (lod === 'micro') {
            ctx.arc(x, y, 4, 0, Math.PI * 2);
        } else if (lod === 'minimal') {
            ctx.arc(x, y, 6, 0, Math.PI * 2);
        } else {
            // Match the rectangular shape of the system box
            const boxWidth = lod === 'simple' ? 40 : 50;
            const boxHeight = lod === 'simple' ? 10 : 14;
            const padding = 3;
            ctx.roundRect(x - boxWidth / 2 - padding, y - boxHeight / 2 - padding, boxWidth + padding * 2, boxHeight + padding * 2, 4);
        }
        ctx.stroke();
        ctx.globalAlpha = 1;
    }

    if (lod === 'micro') {
        // Tiny colored dot at very low zoom
        ctx.fillStyle = secColor;
        ctx.globalAlpha = isHovered ? 0.9 : 0.6;
        ctx.beginPath();
        ctx.arc(x, y, isHovered ? 2 : 1, 0, Math.PI * 2);
        ctx.fill();
        ctx.globalAlpha = 1;
        return;
    }

    if (lod === 'minimal') {
        // Small colored dot
        ctx.fillStyle = secColor;
        ctx.globalAlpha = isHovered ? 0.9 : 0.7;
        ctx.beginPath();
        ctx.arc(x, y, isHovered ? 4 : 2.5, 0, Math.PI * 2);
        ctx.fill();
        ctx.globalAlpha = 1;
        return;
    }

    const boxWidth = lod === 'simple' ? 40 : 50;
    const boxHeight = lod === 'simple' ? 10 : 14;
    const left = x - boxWidth / 2;
    const top = y - boxHeight / 2;

    // Draw connection arrows FIRST (underneath the box)
    if (lod === 'detailed') {
        drawConnectionArrows(ctx, system.id, x, y, boxWidth, boxHeight);
    }

    // Background (sleek neutral styling) - drawn on top of arrows
    ctx.fillStyle = isHovered ? '#262626' : '#171717'; // neutral-800/900
    ctx.strokeStyle = isHovered ? '#525252' : '#404040'; // neutral-600/700
    ctx.lineWidth = isHovered ? 1 : 0.5;
    ctx.beginPath();
    ctx.roundRect(left, top, boxWidth, boxHeight, 2);
    ctx.fill();
    ctx.stroke();

    // Security status indicator (small left badge)
    const secBadgeWidth = lod === 'simple' ? 8 : 10;
    ctx.fillStyle = secColor;
    ctx.beginPath();
    ctx.roundRect(left, top, secBadgeWidth, boxHeight, [2, 0, 0, 2]);
    ctx.fill();

    // Security label inside badge
    ctx.fillStyle = '#000';
    ctx.font = `bold ${lod === 'simple' ? 4 : 5}px system-ui, sans-serif`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(getSecurityLabel(system), left + secBadgeWidth / 2, y);

    if (lod === 'simple') {
        // System name
        ctx.fillStyle = '#a3a3a3'; // neutral-400
        ctx.font = '4px system-ui, sans-serif';
        ctx.textAlign = 'left';
        ctx.fillText(truncate(system.name, 10), left + secBadgeWidth + 2, y);
        return;
    }

    // Detailed view
    const contentLeft = left + secBadgeWidth + 2;
    const iconsRight = left + boxWidth - 2;

    // Draw icons stacked vertically on the right side
    const iconSize = 3;
    const hasIcons = system.has_stations || system.has_belts;
    let iconY = y - (system.has_stations && system.has_belts ? 3 : 0);

    // Station icon (small square)
    if (system.has_stations) {
        ctx.fillStyle = '#60a5fa'; // blue-400
        ctx.fillRect(iconsRight - iconSize, iconY - iconSize / 2, iconSize, iconSize);
        iconY += 6;
    }

    // Belt icon (small diamond)
    if (system.has_belts) {
        ctx.fillStyle = '#fbbf24'; // amber-400
        ctx.beginPath();
        ctx.moveTo(iconsRight - iconSize / 2, iconY - iconSize / 2);
        ctx.lineTo(iconsRight, iconY);
        ctx.lineTo(iconsRight - iconSize / 2, iconY + iconSize / 2);
        ctx.lineTo(iconsRight - iconSize, iconY);
        ctx.closePath();
        ctx.fill();
    }

    // System name
    ctx.fillStyle = '#d4d4d4'; // neutral-300
    ctx.font = '500 5px system-ui, sans-serif';
    ctx.textAlign = 'left';
    ctx.textBaseline = 'middle';

    const maxNameLen = hasIcons ? 10 : 12;
    const sovInfo = getSovInfo(system);
    const sovLogoUrl = getSovLogoUrl(system);

    if (sovInfo && sovLogoUrl) {
        // Name on top
        ctx.fillText(truncate(system.name, maxNameLen), contentLeft, y - 3);

        // Sov logo + name below
        const logoSize = 5;
        const sovY = y + 3;

        // Try to draw logo image
        const logoImg = loadImage(sovLogoUrl);
        if (logoImg) {
            ctx.drawImage(logoImg, contentLeft - 1, sovY - logoSize / 2, logoSize, logoSize);
        } else {
            // Fallback: colored dot while loading
            ctx.fillStyle = sovInfo.type === 'alliance' ? '#60a5fa' : sovInfo.type === 'faction' ? '#fbbf24' : '#a78bfa';
            ctx.beginPath();
            ctx.arc(contentLeft + logoSize / 2 - 1, sovY, logoSize / 2, 0, Math.PI * 2);
            ctx.fill();
        }

        // Sov name
        ctx.fillStyle = '#737373'; // neutral-500
        ctx.font = '4px system-ui, sans-serif';
        ctx.fillText(truncate(sovInfo.name, hasIcons ? 8 : 10), contentLeft + logoSize + 1, sovY);
    } else {
        // Just name centered
        ctx.fillText(truncate(system.name, maxNameLen + 2), contentLeft, y);
    }
}

// Draw arrow lines from center extending outward (system box will overlay the center)
function drawConnectionArrows(ctx: CanvasRenderingContext2D, systemId: number, cx: number, cy: number, boxWidth: number, boxHeight: number) {
    const connections = systemConnectionsMap.value.get(systemId);
    if (!connections || connections.length === 0) return;

    const arrowLength = 6; // How far the arrow extends beyond the box
    const arrowHeadSize = 2.5;

    for (const conn of connections) {
        // Calculate direction to connected system
        const dx = conn.x - cx;
        const dy = conn.y - cy;
        const dist = Math.sqrt(dx * dx + dy * dy);
        if (dist === 0) continue;

        // Normalize direction
        const nx = dx / dist;
        const ny = dy / dist;
        const angle = Math.atan2(dy, dx);

        // Calculate where line exits the box
        const halfW = boxWidth / 2;
        const halfH = boxHeight / 2;
        const absAngle = Math.abs(angle);

        let exitX: number, exitY: number;
        if (absAngle < Math.atan2(halfH, halfW) || absAngle > Math.PI - Math.atan2(halfH, halfW)) {
            exitX = cx + (dx > 0 ? halfW : -halfW);
            exitY = cy + Math.tan(angle) * (dx > 0 ? halfW : -halfW);
            exitY = Math.max(cy - halfH, Math.min(cy + halfH, exitY));
        } else {
            exitY = cy + (dy > 0 ? halfH : -halfH);
            exitX = cx + (dy > 0 ? halfH : -halfH) / Math.tan(angle);
            exitX = Math.max(cx - halfW, Math.min(cx + halfW, exitX));
        }

        // End point of arrow (extends beyond box)
        const endX = exitX + nx * arrowLength;
        const endY = exitY + ny * arrowLength;

        // Draw line from center through to end (box will cover the center part)
        ctx.strokeStyle = conn.regional ? 'rgba(82, 82, 82, 0.35)' : 'rgba(100, 100, 100, 0.4)';
        ctx.lineWidth = 0.8;
        ctx.beginPath();
        ctx.moveTo(cx, cy);
        ctx.lineTo(endX, endY);
        ctx.stroke();

        // Draw arrowhead at the end
        ctx.save();
        ctx.translate(endX, endY);
        ctx.rotate(angle);

        ctx.beginPath();
        ctx.moveTo(0, 0);
        ctx.lineTo(-arrowHeadSize * 1.2, -arrowHeadSize * 0.6);
        ctx.lineTo(-arrowHeadSize * 1.2, arrowHeadSize * 0.6);
        ctx.closePath();

        ctx.fillStyle = conn.regional ? 'rgba(82, 82, 82, 0.4)' : 'rgba(100, 100, 100, 0.5)';
        ctx.fill();

        ctx.restore();
    }
}

function truncate(str: string, len: number): string {
    return str.length > len ? str.slice(0, len - 1) + '…' : str;
}

// Optimized render loop - only draw when needed
let rafId: number | null = null;
let lastDrawTime = 0;
const MIN_FRAME_TIME = 16; // ~60fps max

function requestDraw() {
    needsRedraw.value = true;
}

function renderLoop() {
    if (needsRedraw.value) {
        const now = performance.now();
        if (now - lastDrawTime >= MIN_FRAME_TIME) {
            draw();
            lastDrawTime = now;
            needsRedraw.value = false;
        }
    }
    rafId = requestAnimationFrame(renderLoop);
}

function startRenderLoop() {
    if (!rafId) {
        rafId = requestAnimationFrame(renderLoop);
    }
}

function stopRenderLoop() {
    if (rafId) {
        cancelAnimationFrame(rafId);
        rafId = null;
    }
}

// Handle wheel zoom
function handleWheel(event: WheelEvent) {
    event.preventDefault();

    const rect = containerRef.value?.getBoundingClientRect();
    if (!rect) return;

    const mouseX = event.clientX - rect.left;
    const mouseY = event.clientY - rect.top;

    const zoomFactor = event.deltaY > 0 ? 0.9 : 1.1;
    const newScale = Math.max(0.02, Math.min(5, scale.value * zoomFactor));

    // Zoom towards mouse position
    const scaleRatio = newScale / scale.value;
    const newPanX = mouseX - (mouseX - panOffset.value.x) * scaleRatio;
    const newPanY = mouseY - (mouseY - panOffset.value.y) * scaleRatio;

    scale.value = newScale;
    panOffset.value = { x: newPanX, y: newPanY };
    requestDraw();
}

// Handle mouse down for panning, drawing, or selection
function handleMouseDown(event: MouseEvent) {
    // Prevent default for middle mouse button (browser auto-scroll)
    if (event.button === 1) {
        event.preventDefault();
    }

    if (event.button === 0 || event.button === 1) {
        const rect = containerRef.value?.getBoundingClientRect();
        if (!rect) return;

        const worldX = (event.clientX - rect.left - panOffset.value.x) / scale.value;
        const worldY = (event.clientY - rect.top - panOffset.value.y) / scale.value;

        // Track mouse down position for click detection
        if (event.button === 0) {
            mouseDownPosition.value = { x: event.clientX, y: event.clientY, worldX, worldY };
        }

        // Middle mouse button always pans
        if (event.button === 1) {
            isPanning.value = true;
            lastMousePosition.value = { x: event.clientX, y: event.clientY };
            return;
        }

        if (isDrawingMode.value && event.button === 0) {
            if (drawMode.value === 'eraser') {
                // Delete annotation under cursor
                for (const ann of annotations.value) {
                    if (isPointInAnnotation(worldX, worldY, ann)) {
                        deleteAnnotation(ann.id);
                        break;
                    }
                }
            } else if (drawMode.value === 'rect') {
                // Start drawing rectangle
                isDrawing.value = true;
                drawStart.value = { x: worldX, y: worldY };
                drawCurrent.value = { x: worldX, y: worldY };
            } else if (drawMode.value === 'polygon') {
                // Add point to polygon
                polygonPoints.value.push({ x: worldX, y: worldY });
                drawCurrent.value = { x: worldX, y: worldY }; // Initialize current to prevent line flash
                requestDraw();
            }
        } else if (event.button === 0) {
            // Pan mode
            isPanning.value = true;
            lastMousePosition.value = { x: event.clientX, y: event.clientY };
        }
        event.preventDefault();
    }
}

// Handle double click to finish polygon
function handleDoubleClick(event: MouseEvent) {
    if (isDrawingMode.value && drawMode.value === 'polygon' && polygonPoints.value.length >= 3) {
        const newAnnotation: TAnnotation = {
            id: Date.now().toString(),
            type: 'polygon',
            points: [...polygonPoints.value],
            text: '',
            color: selectedColor.value,
        };
        annotations.value.push(newAnnotation);
        editingAnnotation.value = newAnnotation;
        polygonPoints.value = [];
        requestDraw();
    }
}

// Handle right click context menu
function handleContextMenu(event: MouseEvent) {
    // Always prevent default browser context menu on the canvas
    event.preventDefault();

    // Don't show our context menu in drawing mode
    if (isDrawingMode.value) return;

    const rect = containerRef.value?.getBoundingClientRect();
    if (!rect) return;

    // Transform mouse coordinates to canvas world coordinates (same as handleMouseMove)
    const worldX = (event.clientX - rect.left - panOffset.value.x) / scale.value;
    const worldY = (event.clientY - rect.top - panOffset.value.y) / scale.value;

    // Check if we clicked on a system
    const system = findSystemAtPoint(worldX, worldY);
    if (system) {
        emit('system-contextmenu', system, { x: event.clientX, y: event.clientY });
    }
}

// Handle mouse move for panning, drawing, and hover detection
const handleMouseMove = useThrottleFn((event: MouseEvent) => {
    const rect = containerRef.value?.getBoundingClientRect();
    if (!rect) return;

    const worldX = (event.clientX - rect.left - panOffset.value.x) / scale.value;
    const worldY = (event.clientY - rect.top - panOffset.value.y) / scale.value;

    // Panning takes priority
    if (isPanning.value) {
        const deltaX = event.clientX - lastMousePosition.value.x;
        const deltaY = event.clientY - lastMousePosition.value.y;

        panOffset.value = {
            x: panOffset.value.x + deltaX,
            y: panOffset.value.y + deltaY,
        };

        lastMousePosition.value = { x: event.clientX, y: event.clientY };
        hoveredSystem.value = null;
        hoveredAnnotation.value = null;
        requestDraw();
    } else if (isDrawing.value && isDrawingMode.value && drawMode.value === 'rect') {
        // Update rectangle drawing preview
        drawCurrent.value = { x: worldX, y: worldY };
        requestDraw();
    } else if (isDrawingMode.value && drawMode.value === 'polygon' && polygonPoints.value.length > 0) {
        // Update polygon drawing preview
        drawCurrent.value = { x: worldX, y: worldY };
        requestDraw();
    } else {
        // Check annotation hover (only in intel mode)
        if (isDrawingMode.value) {
            let foundAnnotation: TAnnotation | null = null;
            for (const ann of annotations.value) {
                if (isPointInAnnotation(worldX, worldY, ann)) {
                    foundAnnotation = ann;
                    break;
                }
            }

            if (hoveredAnnotation.value !== foundAnnotation) {
                hoveredAnnotation.value = foundAnnotation;
                requestDraw();
            }
        } else if (hoveredAnnotation.value) {
            hoveredAnnotation.value = null;
            requestDraw();
        }

        // Hit detection for system hover
        const hitRadius = Math.max(30, 15 / scale.value);
        let found: TUniverseSolarsystem | null = null;

        for (const { system, x, y } of visibleSystems.value) {
            const dx = worldX - x;
            const dy = worldY - y;
            if (Math.abs(dx) < hitRadius && Math.abs(dy) < hitRadius / 2) {
                found = system;
                tooltipPosition.value = { x: event.clientX - rect.left, y: event.clientY - rect.top };
                break;
            }
        }

        if (hoveredSystem.value !== found) {
            hoveredSystem.value = found;
            requestDraw();
        }
    }
}, 16);

// Handle mouse up
function handleMouseUp(event: MouseEvent) {
    const wasClick =
        mouseDownPosition.value && Math.abs(event.clientX - mouseDownPosition.value.x) < 5 && Math.abs(event.clientY - mouseDownPosition.value.y) < 5;

    if (isDrawing.value && isDrawingMode.value) {
        // Finish drawing
        isDrawing.value = false;

        const x = Math.min(drawStart.value.x, drawCurrent.value.x);
        const y = Math.min(drawStart.value.y, drawCurrent.value.y);
        const width = Math.abs(drawCurrent.value.x - drawStart.value.x);
        const height = Math.abs(drawCurrent.value.y - drawStart.value.y);

        // Only create annotation if it has some size
        if (width > 10 && height > 10) {
            const newAnnotation: TAnnotation = {
                id: Date.now().toString(),
                type: 'rect',
                x,
                y,
                width,
                height,
                text: '',
                color: selectedColor.value,
            };
            annotations.value.push(newAnnotation);
            editingAnnotation.value = newAnnotation;
        }
        requestDraw();
    } else if (wasClick && !isDrawingMode.value && mouseDownPosition.value) {
        // Check if clicked on a system
        const system = findSystemAtPoint(mouseDownPosition.value.worldX, mouseDownPosition.value.worldY);
        if (system) {
            // Highlight the clicked system
            highlightedRegionId.value = null;
            highlightedConstellationId.value = null;
            highlightedSystemId.value = system.id;
            emit('system-click', system);
            requestDraw();
        } else {
            // Clicked on empty space - clear highlights
            highlightedRegionId.value = null;
            highlightedConstellationId.value = null;
            highlightedSystemId.value = null;
            requestDraw();
        }
    }

    isPanning.value = false;
    mouseDownPosition.value = null;
}

// Center map to fit all content
function centerMap(animate = false) {
    const container = containerRef.value;
    if (!container || props.solarsystems.length === 0) return;

    const rect = container.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) return;

    const wWidth = worldWidth.value;
    const wHeight = worldHeight.value;

    // Validate world dimensions
    if (!isFinite(wWidth) || !isFinite(wHeight) || wWidth <= 0 || wHeight <= 0) {
        return;
    }

    // Calculate scale to fit the entire map
    const scaleX = rect.width / wWidth;
    const scaleY = rect.height / wHeight;
    const fitScale = Math.min(scaleX, scaleY) * 0.9; // 90% to add margin

    const newScale = Math.max(0.02, Math.min(1, fitScale));

    // Center the map
    const centerX = (rect.width - wWidth * newScale) / 2;
    const centerY = (rect.height - wHeight * newScale) / 2;

    if (animate) {
        animateTo(newScale, centerX, centerY);
    } else {
        scale.value = newScale;
        panOffset.value = { x: centerX, y: centerY };
        requestDraw();
    }
}

// Focus on a specific region (zoom and center)
function focusOnRegion(regionId: number | null) {
    // Set highlight state
    highlightedRegionId.value = regionId;
    highlightedConstellationId.value = null;
    highlightedSystemId.value = null;

    const container = containerRef.value;
    if (!container) return;

    const rect = container.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) return;

    // If no region selected, show all
    if (regionId === null) {
        centerMap(true); // Animate to show all
        return;
    }

    // Find all systems in this region
    const regionSystems = systemPositions.value.filter(({ system }) => system.region_id === regionId);

    if (regionSystems.length === 0) return;

    // Calculate bounds of the region
    let minX = Infinity,
        maxX = -Infinity,
        minY = Infinity,
        maxY = -Infinity;
    for (const { x, y } of regionSystems) {
        if (x < minX) minX = x;
        if (x > maxX) maxX = x;
        if (y < minY) minY = y;
        if (y > maxY) maxY = y;
    }

    const regionWidth = maxX - minX;
    const regionHeight = maxY - minY;
    const padding = 100; // Add some padding

    // Calculate scale to fit the region
    const scaleX = rect.width / (regionWidth + padding * 2);
    const scaleY = rect.height / (regionHeight + padding * 2);
    const fitScale = Math.min(scaleX, scaleY) * 0.85;

    const newScale = Math.max(0.05, Math.min(3, fitScale));
    scale.value = newScale;

    // Center on the region
    const regionCenterX = (minX + maxX) / 2;
    const regionCenterY = (minY + maxY) / 2;

    const centerX = rect.width / 2 - regionCenterX * newScale;
    const centerY = rect.height / 2 - regionCenterY * newScale;

    animateTo(newScale, centerX, centerY);
}

// Focus on a specific constellation (zoom and center)
function focusOnConstellation(constellationId: number) {
    // Set highlight state
    highlightedRegionId.value = null;
    highlightedConstellationId.value = constellationId;
    highlightedSystemId.value = null;

    const container = containerRef.value;
    if (!container) return;

    const rect = container.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) return;

    // Find all systems in this constellation
    const constellationSystems = systemPositions.value.filter(({ system }) => system.constellation_id === constellationId);

    if (constellationSystems.length === 0) return;

    // Calculate bounds of the constellation
    let minX = Infinity,
        maxX = -Infinity,
        minY = Infinity,
        maxY = -Infinity;
    for (const { x, y } of constellationSystems) {
        if (x < minX) minX = x;
        if (x > maxX) maxX = x;
        if (y < minY) minY = y;
        if (y > maxY) maxY = y;
    }

    const constellationWidth = maxX - minX;
    const constellationHeight = maxY - minY;
    const padding = 50;

    // Calculate scale to fit the constellation
    const scaleX = rect.width / (constellationWidth + padding * 2);
    const scaleY = rect.height / (constellationHeight + padding * 2);
    const fitScale = Math.min(scaleX, scaleY) * 0.85;

    const newScale = Math.max(0.1, Math.min(5, fitScale));
    scale.value = newScale;

    // Center on the constellation
    const centerX = rect.width / 2 - ((minX + maxX) / 2) * newScale;
    const centerY = rect.height / 2 - ((minY + maxY) / 2) * newScale;

    animateTo(newScale, centerX, centerY);
}

// Focus on a specific system (zoom and center)
function focusOnSystem(systemId: number) {
    // Set highlight state
    highlightedRegionId.value = null;
    highlightedConstellationId.value = null;
    highlightedSystemId.value = systemId;

    const container = containerRef.value;
    if (!container) return;

    const rect = container.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) return;

    // Find the system
    const systemData = systemPositions.value.find(({ system }) => system.id === systemId);

    if (!systemData) return;

    // Find all systems in the same constellation to determine zoom level
    const constellationSystems = systemPositions.value.filter(({ system }) => system.constellation_id === systemData.system.constellation_id);

    // Calculate bounds of the constellation
    let minX = Infinity,
        maxX = -Infinity,
        minY = Infinity,
        maxY = -Infinity;
    for (const { x, y } of constellationSystems) {
        if (x < minX) minX = x;
        if (x > maxX) maxX = x;
        if (y < minY) minY = y;
        if (y > maxY) maxY = y;
    }

    const constellationWidth = maxX - minX;
    const constellationHeight = maxY - minY;
    const padding = 50;

    // Calculate scale to fit the constellation (same as focusOnConstellation)
    const scaleX = rect.width / (constellationWidth + padding * 2);
    const scaleY = rect.height / (constellationHeight + padding * 2);
    const fitScale = Math.min(scaleX, scaleY) * 0.85;

    const newScale = Math.max(0.1, Math.min(5, fitScale));

    // Center on the specific system (not constellation center)
    const centerX = rect.width / 2 - systemData.x * newScale;
    const centerY = rect.height / 2 - systemData.y * newScale;

    animateTo(newScale, centerX, centerY);
}

// Draw annotations on the canvas
function drawAnnotations(ctx: CanvasRenderingContext2D) {
    // Determine LOD for annotations
    const annLod: 'minimal' | 'simple' | 'detailed' = scale.value < 0.08 ? 'minimal' : scale.value < 0.25 ? 'simple' : 'detailed';

    // Draw saved annotations
    for (const ann of annotations.value) {
        const isHovered = isDrawingMode.value && hoveredAnnotation.value?.id === ann.id;
        const center = getAnnotationCenter(ann);

        // Minimal LOD: just a colored marker
        if (annLod === 'minimal') {
            ctx.fillStyle = ann.color;
            ctx.beginPath();
            ctx.arc(center.x, center.y, 8 / scale.value, 0, Math.PI * 2);
            ctx.fill();
            continue;
        }

        const lineWidth = 2 / scale.value;

        // Measure text (only at detailed LOD)
        let textWidth = 0;
        const labelPadding = 6 / scale.value;
        const fontSize = Math.max(10, 14 / scale.value);

        if (ann.text && annLod === 'detailed') {
            ctx.font = `600 ${fontSize}px system-ui, sans-serif`;
            textWidth = ctx.measureText(ann.text).width + labelPadding * 2;
        }

        // Semi-transparent fill
        ctx.fillStyle = ann.color + (isHovered ? '18' : '12');
        ctx.beginPath();

        if (ann.type === 'rect' && ann.x !== undefined && ann.y !== undefined && ann.width !== undefined && ann.height !== undefined) {
            ctx.rect(ann.x, ann.y, ann.width, ann.height);
        } else if (ann.type === 'polygon' && ann.points && ann.points.length >= 3) {
            ctx.moveTo(ann.points[0].x, ann.points[0].y);
            for (let i = 1; i < ann.points.length; i++) {
                ctx.lineTo(ann.points[i].x, ann.points[i].y);
            }
            ctx.closePath();
        }
        ctx.fill();

        // Draw dashed border
        ctx.strokeStyle = ann.color;
        ctx.lineWidth = lineWidth;
        ctx.setLineDash([8 / scale.value, 4 / scale.value]);

        if (ann.type === 'rect' && ann.x !== undefined && ann.y !== undefined && ann.width !== undefined && ann.height !== undefined) {
            if (ann.text && textWidth > 0 && annLod === 'detailed') {
                const gapStart = center.x - textWidth / 2;
                const gapEnd = center.x + textWidth / 2;

                ctx.beginPath();
                ctx.moveTo(ann.x, ann.y);
                ctx.lineTo(gapStart, ann.y);
                ctx.moveTo(gapEnd, ann.y);
                ctx.lineTo(ann.x + ann.width, ann.y);
                ctx.lineTo(ann.x + ann.width, ann.y + ann.height);
                ctx.lineTo(ann.x, ann.y + ann.height);
                ctx.lineTo(ann.x, ann.y);
                ctx.stroke();
            } else {
                ctx.beginPath();
                ctx.rect(ann.x, ann.y, ann.width, ann.height);
                ctx.stroke();
            }
        } else if (ann.type === 'polygon' && ann.points && ann.points.length >= 3) {
            ctx.beginPath();
            ctx.moveTo(ann.points[0].x, ann.points[0].y);
            for (let i = 1; i < ann.points.length; i++) {
                ctx.lineTo(ann.points[i].x, ann.points[i].y);
            }
            ctx.closePath();
            ctx.stroke();
        }

        ctx.setLineDash([]);

        // Draw text label (only at detailed LOD)
        if (ann.text && annLod === 'detailed') {
            ctx.fillStyle = '#0a0a0a';
            ctx.fillRect(center.x - textWidth / 2, center.y - fontSize / 2 - 2 / scale.value, textWidth, fontSize + 4 / scale.value);

            ctx.fillStyle = ann.color;
            ctx.font = `600 ${fontSize}px system-ui, sans-serif`;
            ctx.textAlign = 'center';
            ctx.textBaseline = 'middle';
            ctx.fillText(ann.text, center.x, center.y);
        }
    }

    // Draw current polygon being drawn
    if (isDrawingMode.value && drawMode.value === 'polygon' && polygonPoints.value.length > 0) {
        ctx.strokeStyle = selectedColor.value;
        ctx.lineWidth = 2 / scale.value;
        ctx.setLineDash([8 / scale.value, 4 / scale.value]);

        ctx.beginPath();
        ctx.moveTo(polygonPoints.value[0].x, polygonPoints.value[0].y);
        for (let i = 1; i < polygonPoints.value.length; i++) {
            ctx.lineTo(polygonPoints.value[i].x, polygonPoints.value[i].y);
        }
        ctx.lineTo(drawCurrent.value.x, drawCurrent.value.y);
        ctx.stroke();
        ctx.setLineDash([]);

        // Draw points
        ctx.fillStyle = selectedColor.value;
        for (const p of polygonPoints.value) {
            ctx.beginPath();
            ctx.arc(p.x, p.y, 4 / scale.value, 0, Math.PI * 2);
            ctx.fill();
        }
    }

    // Draw current drawing preview
    if (isDrawing.value && isDrawingMode.value) {
        const x = Math.min(drawStart.value.x, drawCurrent.value.x);
        const y = Math.min(drawStart.value.y, drawCurrent.value.y);
        const width = Math.abs(drawCurrent.value.x - drawStart.value.x);
        const height = Math.abs(drawCurrent.value.y - drawStart.value.y);

        ctx.fillStyle = selectedColor.value + '30';
        ctx.strokeStyle = selectedColor.value;
        ctx.lineWidth = 2 / scale.value;
        ctx.setLineDash([8 / scale.value, 4 / scale.value]);

        ctx.beginPath();
        ctx.rect(x, y, width, height);
        ctx.fill();
        ctx.stroke();
        ctx.setLineDash([]);
    }
}

// Toggle drawing mode
function toggleDrawingMode() {
    isDrawingMode.value = !isDrawingMode.value;
    if (!isDrawingMode.value) {
        isDrawing.value = false;
        polygonPoints.value = [];
    }
}

// Delete an annotation
function deleteAnnotation(id: string) {
    annotations.value = annotations.value.filter((a) => a.id !== id);
    requestDraw();
}

// Find system at canvas world coordinates (after Y inversion)
function findSystemAtPoint(canvasWorldX: number, canvasWorldY: number): TUniverseSolarsystem | null {
    // System box dimensions (must match drawSolarsystem)
    const lod: 'micro' | 'minimal' | 'simple' | 'detailed' =
        scale.value < 0.03 ? 'micro' : scale.value < 0.08 ? 'minimal' : scale.value < 0.25 ? 'simple' : 'detailed';

    // At micro/minimal LOD, use a larger hit area
    const hitRadius = lod === 'micro' ? 15 : lod === 'minimal' ? 10 : 0;
    const boxWidth = lod === 'simple' ? 40 : 50;
    const boxHeight = lod === 'simple' ? 10 : 14;
    const halfW = boxWidth / 2;
    const halfH = boxHeight / 2;

    // Use transformed positions (which have Y inverted)
    for (const { system, x, y } of systemPositions.value) {
        if (hitRadius > 0) {
            // Circle hit test for minimal views
            const dx = canvasWorldX - x;
            const dy = canvasWorldY - y;
            if (dx * dx + dy * dy <= hitRadius * hitRadius) {
                return system;
            }
        } else {
            // Rectangle hit test for detailed views
            if (canvasWorldX >= x - halfW && canvasWorldX <= x + halfW && canvasWorldY >= y - halfH && canvasWorldY <= y + halfH) {
                return system;
            }
        }
    }
    return null;
}

// Clear all annotations
function clearAnnotations() {
    annotations.value = [];
    requestDraw();
}

// Save annotation text
function saveAnnotationText(text: string) {
    if (editingAnnotation.value) {
        editingAnnotation.value.text = text;
        editingAnnotation.value = null;
        requestDraw();
    }
}

// Cancel annotation editing
function cancelAnnotationEdit() {
    if (editingAnnotation.value && !editingAnnotation.value.text) {
        // Remove annotation if no text was added
        deleteAnnotation(editingAnnotation.value.id);
    }
    editingAnnotation.value = null;
}

// Handle keyboard shortcuts with useMagicKeys
const { escape } = useMagicKeys();

whenever(escape, () => {
    if (isDrawingMode.value) {
        // First check if there's an active drawing to cancel
        if (isDrawing.value || polygonPoints.value.length > 0) {
            // Cancel current drawing but stay in intel mode
            polygonPoints.value = [];
            isDrawing.value = false;
            requestDraw();
        } else {
            // No active drawing, exit intel mode
            isDrawingMode.value = false;
        }
    }
});

// Expose methods for parent components
// Clear system selection/highlight
function clearSystemSelection() {
    highlightedSystemId.value = null;
    requestDraw();
}

// Animated zoom towards center of screen
function zoomIn() {
    const newScale = Math.min(5, scale.value * 1.2);
    const centerX = containerWidth.value / 2;
    const centerY = containerHeight.value / 2;
    
    const scaleRatio = newScale / scale.value;
    const newPanX = centerX - (centerX - panOffset.value.x) * scaleRatio;
    const newPanY = centerY - (centerY - panOffset.value.y) * scaleRatio;
    
    animateTo(newScale, newPanX, newPanY, 150);
}

function zoomOut() {
    const newScale = Math.max(0.02, scale.value / 1.2);
    const centerX = containerWidth.value / 2;
    const centerY = containerHeight.value / 2;
    
    const scaleRatio = newScale / scale.value;
    const newPanX = centerX - (centerX - panOffset.value.x) * scaleRatio;
    const newPanY = centerY - (centerY - panOffset.value.y) * scaleRatio;
    
    animateTo(newScale, newPanX, newPanY, 150);
}

defineExpose({
    centerMap,
    focusOnRegion,
    focusOnConstellation,
    focusOnSystem,
    clearSystemSelection,
    zoomIn,
    zoomOut,
    toggleDrawingMode,
    clearAnnotations,
    isDrawingMode,
    drawMode,
    annotations,
    selectedColor,
    annotationColors,
});

// Track if we've centered the map initially
const hasCentered = ref(false);

// Watch for changes that require redraw
watch([containerWidth, containerHeight, scale, panOffset], () => requestDraw());

// Focus intel input when editing annotation
watch(editingAnnotation, (ann) => {
    if (ann) {
        // Use nextTick to ensure the input is rendered
        setTimeout(() => {
            intelInputRef.value?.focus();
        }, 50);
    }
});

// Preload sovereignty logos for visible systems
watch(
    visibleSystems,
    (systems) => {
        for (const { system } of systems) {
            const url = getSovLogoUrl(system);
            if (url && !imageCache.has(url)) {
                loadImage(url);
            }
        }
    },
    { immediate: true },
);

// Initialize
onMounted(() => {
    loadAnnotations();
    startRenderLoop();
    requestDraw();
});

onUnmounted(() => {
    stopRenderLoop();
});

// Center map when container has size AND we have systems
watch(
    [containerWidth, containerHeight, () => props.solarsystems.length],
    ([width, height, len]) => {
        if (!hasCentered.value && width > 0 && height > 0 && len > 0) {
            centerMap();
            hasCentered.value = true;
        }
    },
    { immediate: true },
);
</script>

<template>
    <div
        ref="container"
        class="relative h-full w-full overflow-hidden"
        :class="[
            isDrawingMode
                ? drawMode === 'eraser'
                    ? hoveredAnnotation
                        ? 'cursor-pointer'
                        : 'cursor-crosshair'
                    : 'cursor-crosshair'
                : isPanning
                  ? 'cursor-grabbing'
                  : 'cursor-grab',
        ]"
        @wheel.prevent="handleWheel"
        @mousedown="handleMouseDown"
        @mousemove="handleMouseMove"
        @mouseup="handleMouseUp"
        @mouseleave="handleMouseUp"
        @dblclick="handleDoubleClick"
        @auxclick.prevent
        @contextmenu.prevent="handleContextMenu"
    >
        <!-- Canvas -->
        <canvas ref="canvas" class="block h-full w-full" />

        <!-- Controls hint -->
        <div class="absolute right-4 bottom-4 rounded-lg bg-black/60 px-3 py-2 text-xs text-neutral-400 backdrop-blur-sm">
            <div class="flex items-center gap-2">
                <span>Scroll to zoom</span>
                <span class="text-neutral-600">|</span>
                <span>Drag to pan</span>
            </div>
        </div>

        <!-- Drawing mode indicator -->
        <div
            v-if="isDrawingMode"
            class="absolute top-4 left-1/2 flex -translate-x-1/2 items-center gap-2 rounded-lg px-4 py-2 text-sm font-medium text-white backdrop-blur-sm"
            :class="drawMode === 'eraser' ? 'bg-red-500/90' : 'bg-blue-500/90'"
        >
            <Square v-if="drawMode === 'rect'" class="h-4 w-4" />
            <PenTool v-else-if="drawMode === 'polygon'" class="h-4 w-4" />
            <Eraser v-else class="h-4 w-4" />
            <span v-if="drawMode === 'rect'">Click and drag to draw a rectangle</span>
            <span v-else-if="drawMode === 'polygon'">Click to add points, double-click to finish</span>
            <span v-else>Click on an annotation to delete it</span>
        </div>

        <!-- Annotation text input popup -->
        <div
            v-if="editingAnnotation"
            class="absolute top-1/2 left-1/2 z-50 -translate-x-1/2 -translate-y-1/2 rounded-xl border border-neutral-700 bg-neutral-900 p-4 shadow-2xl"
            @mousedown.stop
            @mouseup.stop
            @click.stop
        >
            <div class="mb-3 text-sm font-medium text-neutral-200">Add Intel Note</div>
            <input
                ref="intelInputRef"
                type="text"
                v-model="editingAnnotation.text"
                @keyup.enter="saveAnnotationText(editingAnnotation.text)"
                @keyup.escape="cancelAnnotationEdit"
                class="w-64 rounded-lg border border-neutral-600 bg-neutral-800 px-3 py-2 text-sm text-white placeholder-neutral-500 focus:border-blue-500 focus:outline-none"
                placeholder="Enter intel note..."
            />
            <div class="mt-3 flex justify-end gap-2">
                <button @click="cancelAnnotationEdit" class="rounded-lg px-3 py-1.5 text-xs text-neutral-400 hover:bg-neutral-800">Cancel</button>
                <button
                    @click="saveAnnotationText(editingAnnotation.text)"
                    class="rounded-lg bg-blue-600 px-3 py-1.5 text-xs text-white hover:bg-blue-500"
                >
                    Save
                </button>
            </div>
        </div>
    </div>
</template>
