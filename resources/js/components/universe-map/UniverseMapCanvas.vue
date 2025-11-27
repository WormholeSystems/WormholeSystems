<script setup lang="ts">
import { TUniverseBounds, TUniverseConnection, TUniverseSolarsystem } from '@/types/universe-map';
import { useElementSize, useThrottleFn } from '@vueuse/core';
import { Delaunay } from 'd3-delaunay';
import { computed, onMounted, onUnmounted, ref, shallowRef, useTemplateRef, watch } from 'vue';

const props = defineProps<{
    solarsystems: TUniverseSolarsystem[];
    connections: TUniverseConnection[];
    bounds: TUniverseBounds;
}>();

const scale = defineModel<number>('scale', { default: 0.5 });
const panOffset = defineModel<{ x: number; y: number }>('panOffset', { default: () => ({ x: 0, y: 0 }) });

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
const { width: containerWidth, height: containerHeight } = useElementSize(containerRef);

// Track hovered system for tooltip
const hoveredSystem = shallowRef<TUniverseSolarsystem | null>(null);
const tooltipPosition = ref({ x: 0, y: 0 });

// Panning state
const isPanning = ref(false);
const lastMousePosition = ref({ x: 0, y: 0 });

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

    // Draw region labels last (overlay everything)
    drawRegionLabels(ctx, lod);

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

// Draw a single system
function drawSystem(
    ctx: CanvasRenderingContext2D,
    system: TUniverseSolarsystem,
    x: number,
    y: number,
    lod: 'micro' | 'minimal' | 'simple' | 'detailed',
) {
    const isHovered = hoveredSystem.value?.id === system.id;
    const secColor = getSecurityColor(system);

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

// Handle mouse down for panning
function handleMouseDown(event: MouseEvent) {
    if (event.button === 0 || event.button === 1) {
        isPanning.value = true;
        lastMousePosition.value = { x: event.clientX, y: event.clientY };
        event.preventDefault();
    }
}

// Handle mouse move for panning and hover detection
const handleMouseMove = useThrottleFn((event: MouseEvent) => {
    if (isPanning.value) {
        const deltaX = event.clientX - lastMousePosition.value.x;
        const deltaY = event.clientY - lastMousePosition.value.y;

        panOffset.value = {
            x: panOffset.value.x + deltaX,
            y: panOffset.value.y + deltaY,
        };

        lastMousePosition.value = { x: event.clientX, y: event.clientY };
        hoveredSystem.value = null;
        requestDraw();
    } else {
        // Hit detection for hover
        const rect = containerRef.value?.getBoundingClientRect();
        if (!rect) return;

        const mouseX = (event.clientX - rect.left - panOffset.value.x) / scale.value;
        const mouseY = (event.clientY - rect.top - panOffset.value.y) / scale.value;

        // Find system under cursor (adaptive hit radius based on zoom)
        const hitRadius = Math.max(30, 15 / scale.value);
        let found: TUniverseSolarsystem | null = null;

        for (const { system, x, y } of visibleSystems.value) {
            const dx = mouseX - x;
            const dy = mouseY - y;
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
function handleMouseUp() {
    isPanning.value = false;
}

// Center map to fit all content
function centerMap() {
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
    scale.value = newScale;

    // Center the map
    const centerX = (rect.width - wWidth * newScale) / 2;
    const centerY = (rect.height - wHeight * newScale) / 2;

    panOffset.value = { x: centerX, y: centerY };
    requestDraw();
}

// Focus on a specific region (zoom and center)
function focusOnRegion(regionId: number | null) {
    const container = containerRef.value;
    if (!container) return;

    const rect = container.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) return;

    // If no region selected, show all
    if (regionId === null) {
        centerMap();
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

    panOffset.value = { x: centerX, y: centerY };
    requestDraw();
}

// Expose methods for parent components
defineExpose({
    centerMap,
    focusOnRegion,
});

// Track if we've centered the map initially
const hasCentered = ref(false);

// Watch for changes that require redraw
watch([containerWidth, containerHeight], () => requestDraw());

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
        class="relative h-full w-full cursor-grab overflow-hidden active:cursor-grabbing"
        @wheel.prevent="handleWheel"
        @mousedown="handleMouseDown"
        @mousemove="handleMouseMove"
        @mouseup="handleMouseUp"
        @mouseleave="handleMouseUp"
    >
        <!-- Canvas -->
        <canvas ref="canvas" class="block h-full w-full" />

        <!-- Tooltip -->
        <div
            v-if="hoveredSystem"
            class="pointer-events-none absolute z-50 max-w-xs rounded-lg border border-neutral-700 bg-neutral-900/95 px-3 py-2 text-xs shadow-xl backdrop-blur-sm"
            :style="{
                left: Math.min(tooltipPosition.x + 12, containerWidth - 200) + 'px',
                top: Math.min(tooltipPosition.y + 12, containerHeight - 100) + 'px',
            }"
        >
            <div class="font-semibold text-white">{{ hoveredSystem.name }}</div>
            <div class="mt-1 text-neutral-400">
                <span v-if="hoveredSystem.class" class="text-cyan-400">C{{ hoveredSystem.class }}</span>
                <span
                    v-else
                    :class="hoveredSystem.security >= 0.5 ? 'text-green-500' : hoveredSystem.security >= 0.1 ? 'text-orange-500' : 'text-red-500'"
                >
                    {{ hoveredSystem.security.toFixed(2) }}
                </span>
                <span class="mx-1">•</span>
                {{ hoveredSystem.constellation.name }}
            </div>
            <div class="text-neutral-500">{{ hoveredSystem.region.name }}</div>
            <div v-if="hoveredSystem.sovereignty?.alliance" class="mt-1 text-blue-400">
                {{ hoveredSystem.sovereignty.alliance.name }} [{{ hoveredSystem.sovereignty.alliance.ticker }}]
            </div>
            <div v-else-if="hoveredSystem.sovereignty?.faction" class="mt-1 text-amber-400">
                {{ hoveredSystem.sovereignty.faction.name }}
            </div>
        </div>

        <!-- Controls hint -->
        <div class="absolute right-4 bottom-4 rounded-lg bg-black/60 px-3 py-2 text-xs text-neutral-400 backdrop-blur-sm">
            <div class="flex items-center gap-2">
                <span>Scroll to zoom</span>
                <span class="text-neutral-600">|</span>
                <span>Drag to pan</span>
            </div>
        </div>
    </div>
</template>
