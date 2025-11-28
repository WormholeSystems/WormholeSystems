/**
 * CSS variable names for security colors (use in DOM elements)
 */
export const CSS_VARS = {
    hs: 'var(--color-hs)',
    ls: 'var(--color-ls)',
    ns: 'var(--color-ns)',
    c1: 'var(--color-c1)',
    c2: 'var(--color-c2)',
    c3: 'var(--color-c3)',
    c4: 'var(--color-c4)',
    c5: 'var(--color-c5)',
    c6: 'var(--color-c6)',
    friendly: 'var(--color-friendly)',
    hostile: 'var(--color-hostile)',
    unknown: 'var(--color-unknown)',
    active: 'var(--color-active)',
    empty: 'var(--color-empty)',
    unscanned: 'var(--color-unscanned)',
} as const;

/**
 * Resolve a CSS variable to its actual color value (for canvas rendering)
 * Uses getComputedStyle for one-off lookups. For reactive usage, use useCssVar from @vueuse/core
 */
export function resolveCssVar(varName: string): string {
    if (typeof document === 'undefined') return '#888888';

    // Extract variable name from var(--name) format
    const match = varName.match(/var\(--([^)]+)\)/);
    if (match) {
        const value = getComputedStyle(document.documentElement).getPropertyValue(`--${match[1]}`).trim();
        return value || '#888888';
    }

    return varName;
}

/**
 * Extract CSS variable name from var(--name) format
 */
export function extractCssVarName(varString: string): string | null {
    const match = varString.match(/var\(--([^)]+)\)/);
    return match ? match[1] : null;
}

/**
 * Wormhole class colors mapping
 */
const WH_CLASS_COLORS: Record<number, string> = {
    1: 'var(--color-c1)',
    2: 'var(--color-c2)',
    3: 'var(--color-c3)',
    4: 'var(--color-c4)',
    5: 'var(--color-c5)',
    6: 'var(--color-c6)',
    13: 'var(--color-hs)', // Shattered uses HS color
};

/**
 * Get security status CSS variable for a system
 */
export function getSecurityCssVar(security: number, wormholeClass: number | null): string {
    if (wormholeClass !== null) {
        return WH_CLASS_COLORS[wormholeClass] ?? 'var(--color-muted-foreground)';
    }

    if (security >= 0.5) return CSS_VARS.hs;
    if (security >= 0.1) return CSS_VARS.ls;
    return CSS_VARS.ns;
}

/**
 * Get security status color for a system (resolved for canvas)
 */
export function getSecurityColor(security: number, wormholeClass: number | null): string {
    return resolveCssVar(getSecurityCssVar(security, wormholeClass));
}

/**
 * Get security status text color class (Tailwind class)
 */
export function getSecurityTextClass(security: number, wormholeClass: number | null): string {
    if (wormholeClass !== null) return 'text-c1'; // Uses custom color class
    if (security >= 0.5) return 'text-hs';
    if (security >= 0.1) return 'text-ls';
    return 'text-ns';
}

/**
 * Get system class string (HS/LS/NS or C1-C6)
 */
export function getSystemClassString(security: number, wormholeClass: number | null): string {
    if (wormholeClass !== null) return `C${wormholeClass}`;
    if (security >= 0.5) return 'HS';
    if (security > 0) return 'LS';
    return 'NS';
}

/**
 * Format security status number
 */
export function formatSecurity(security: number): string {
    return security.toFixed(2);
}

/**
 * Calculate Level of Detail based on scale
 */
export type LODLevel = 'micro' | 'minimal' | 'simple' | 'detailed';

export function getLODLevel(scale: number): LODLevel {
    if (scale < 0.03) return 'micro';
    if (scale < 0.08) return 'minimal';
    if (scale < 0.25) return 'simple';
    return 'detailed';
}

/**
 * Color palette for regions (RGB values for canvas, matches Tailwind CSS colors)
 * These are RGB values that can be used with rgba() for transparency
 */
export const REGION_PALETTE_RGB = [
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

/**
 * Region palette as CSS variable references
 */
export const REGION_PALETTE_VARS = [
    'var(--color-red-500)',
    'var(--color-orange-500)',
    'var(--color-yellow-500)',
    'var(--color-lime-500)',
    'var(--color-green-500)',
    'var(--color-emerald-500)',
    'var(--color-teal-500)',
    'var(--color-cyan-500)',
    'var(--color-sky-500)',
    'var(--color-blue-500)',
    'var(--color-indigo-500)',
    'var(--color-violet-500)',
    'var(--color-purple-500)',
    'var(--color-fuchsia-500)',
    'var(--color-pink-500)',
    'var(--color-rose-500)',
];

/**
 * Get a consistent RGB color string for a region (for canvas rgba())
 */
export function getRegionColorRGB(regionId: number): string {
    return REGION_PALETTE_RGB[regionId % REGION_PALETTE_RGB.length];
}

/**
 * Get a consistent CSS variable for a region
 */
export function getRegionColorVar(regionId: number): string {
    return REGION_PALETTE_VARS[regionId % REGION_PALETTE_VARS.length];
}

/**
 * @deprecated Use getRegionColorRGB instead
 */
export const REGION_PALETTE = REGION_PALETTE_RGB;

/**
 * @deprecated Use getRegionColorRGB instead
 */
export function getRegionColor(regionId: number): string {
    return getRegionColorRGB(regionId);
}

/**
 * Generate sovereignty logo URL
 */
export function getSovLogoUrl(
    sovereignty: { alliance?: { id: number } | null; corporation?: { id: number } | null; faction?: { id: number } | null } | null,
): string | null {
    if (sovereignty?.alliance) {
        return `https://images.evetech.net/alliances/${sovereignty.alliance.id}/logo?size=32`;
    }
    if (sovereignty?.corporation) {
        return `https://images.evetech.net/corporations/${sovereignty.corporation.id}/logo?size=32`;
    }
    if (sovereignty?.faction) {
        return `https://images.evetech.net/corporations/${sovereignty.faction.id}/logo?size=32`;
    }
    return null;
}

/**
 * Truncate text with ellipsis
 */
export function truncateText(ctx: CanvasRenderingContext2D, text: string, maxWidth: number): string {
    const metrics = ctx.measureText(text);
    if (metrics.width <= maxWidth) return text;

    let truncated = text;
    while (truncated.length > 0 && ctx.measureText(truncated + '…').width > maxWidth) {
        truncated = truncated.slice(0, -1);
    }
    return truncated + '…';
}

/**
 * Calculate bounds from a list of points
 */
export function calculateBounds(points: { x: number; y: number }[]): { minX: number; maxX: number; minY: number; maxY: number } {
    let minX = Infinity,
        maxX = -Infinity,
        minY = Infinity,
        maxY = -Infinity;

    for (const p of points) {
        if (p.x < minX) minX = p.x;
        if (p.x > maxX) maxX = p.x;
        if (p.y < minY) minY = p.y;
        if (p.y > maxY) maxY = p.y;
    }

    return { minX, maxX, minY, maxY };
}

/**
 * Calculate centroid of points
 */
export function calculateCentroid(points: { x: number; y: number }[]): { x: number; y: number } {
    if (points.length === 0) return { x: 0, y: 0 };

    let sumX = 0,
        sumY = 0;
    for (const p of points) {
        sumX += p.x;
        sumY += p.y;
    }

    return { x: sumX / points.length, y: sumY / points.length };
}

/**
 * Easing functions for animations
 */
export const easing = {
    linear: (t: number) => t,
    easeInQuad: (t: number) => t * t,
    easeOutQuad: (t: number) => t * (2 - t),
    easeInOutQuad: (t: number) => (t < 0.5 ? 2 * t * t : -1 + (4 - 2 * t) * t),
    easeOutCubic: (t: number) => 1 - Math.pow(1 - t, 3),
    easeInOutCubic: (t: number) => (t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2),
};
