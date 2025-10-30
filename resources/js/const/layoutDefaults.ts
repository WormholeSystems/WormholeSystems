import { BreakpointsConfig } from '@/types/layout';

/**
 * Default breakpoint configurations
 * These define the standard responsive breakpoints and their layouts
 */
export const DEFAULT_BREAKPOINTS: BreakpointsConfig = {
    xs: {
        key: 'xs',
        label: 'Extra Small',
        minWidth: 0,
        description: 'Mobile',
        cols: 1,
        rowHeight: 100,
        items: [
            { i: 'map', x: 0, y: 0, w: 1, h: 8, minH: 3, static: false },
            { i: 'solarsystem', x: 0, y: 8, w: 1, h: 4, minH: 2, static: false },
            { i: 'signatures', x: 0, y: 12, w: 1, h: 3, minH: 2, static: false },
            { i: 'audits', x: 0, y: 15, w: 1, h: 2, minH: 1, static: false },
            { i: 'ship-history', x: 0, y: 17, w: 1, h: 2, minH: 1, static: false },
            { i: 'characters', x: 0, y: 19, w: 1, h: 2, minH: 1, static: false },
            { i: 'killmails', x: 0, y: 21, w: 1, h: 3, minH: 2, static: false },
            { i: 'autopilot', x: 0, y: 24, w: 1, h: 4, minH: 2, static: false },
        ],
    },
    sm: {
        key: 'sm',
        label: 'Small',
        minWidth: 640,
        description: 'Mobile & Tablet',
        cols: 2,
        rowHeight: 100,
        items: [
            { i: 'map', x: 0, y: 0, w: 2, h: 8, minH: 3, static: false },
            { i: 'solarsystem', x: 0, y: 8, w: 2, h: 4, minH: 2, static: false },
            { i: 'signatures', x: 0, y: 12, w: 2, h: 3, minH: 2, static: false },
            { i: 'audits', x: 0, y: 15, w: 1, h: 2, minH: 1, static: false },
            { i: 'ship-history', x: 1, y: 15, w: 1, h: 2, minH: 1, static: false },
            { i: 'characters', x: 0, y: 17, w: 1, h: 2, minH: 1, static: false },
            { i: 'killmails', x: 1, y: 17, w: 1, h: 3, minH: 2, static: false },
            { i: 'autopilot', x: 0, y: 19, w: 2, h: 4, minH: 2, static: false },
        ],
    },
    md: {
        key: 'md',
        label: 'Medium',
        minWidth: 1024,
        description: 'Tablet & Small Desktop',
        cols: 4,
        rowHeight: 100,
        items: [
            { i: 'map', x: 0, y: 0, w: 4, h: 8, minH: 3, static: false },
            { i: 'solarsystem', x: 0, y: 8, w: 2, h: 4, minH: 2, static: false },
            { i: 'signatures', x: 0, y: 12, w: 2, h: 3, minH: 2, static: false },
            { i: 'audits', x: 0, y: 15, w: 2, h: 2, minH: 1, static: false },
            { i: 'ship-history', x: 0, y: 17, w: 2, h: 2, minH: 1, static: false },
            { i: 'characters', x: 2, y: 8, w: 2, h: 2, minH: 1, static: false },
            { i: 'killmails', x: 2, y: 10, w: 2, h: 3, minH: 2, static: false },
            { i: 'autopilot', x: 2, y: 13, w: 2, h: 4, minH: 2, static: false },
        ],
    },
    lg: {
        key: 'lg',
        label: 'Large',
        minWidth: 1536,
        description: 'Desktop & Wide Screen',
        cols: 10,
        rowHeight: 100,
        items: [
            { i: 'map', x: 0, y: 0, w: 10, h: 8, minH: 3, static: false },
            { i: 'solarsystem', x: 0, y: 8, w: 4, h: 4, minH: 2, static: false },
            { i: 'signatures', x: 0, y: 12, w: 4, h: 3, minH: 2, static: false },
            { i: 'audits', x: 0, y: 15, w: 4, h: 2, minH: 1, static: false },
            { i: 'ship-history', x: 0, y: 17, w: 4, h: 2, minH: 1, static: false },
            { i: 'characters', x: 4, y: 8, w: 3, h: 2, minH: 1, static: false },
            { i: 'killmails', x: 4, y: 10, w: 3, h: 3, minH: 2, static: false },
            { i: 'autopilot', x: 7, y: 8, w: 3, h: 4, minH: 2, static: false },
        ],
    },
};

/**
 * List of protected breakpoint keys that cannot be deleted
 */
export const PROTECTED_BREAKPOINTS = ['xs', 'sm', 'md', 'lg'] as const;

/**
 * Check if a breakpoint key is protected (default)
 */
export function isProtectedBreakpoint(key: string): boolean {
    return PROTECTED_BREAKPOINTS.includes(key as any);
}
