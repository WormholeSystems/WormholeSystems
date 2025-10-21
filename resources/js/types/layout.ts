export type Breakpoint = 'sm' | 'md' | 'lg';

export interface LayoutItem {
    i: string; // Component identifier
    x: number; // X position in grid units
    y: number; // Y position in grid units
    w: number; // Width in grid units
    h: number; // Height in grid units
    minW?: number; // Minimum width
    minH?: number; // Minimum height
    maxW?: number; // Maximum width
    maxH?: number; // Maximum height
    static?: boolean; // Whether item is draggable/resizable
}

export interface LayoutConfig {
    cols: number; // Number of columns
    rowHeight: number; // Height of each row in pixels
    items: LayoutItem[];
}

export type LayoutConfigs = {
    sm: LayoutConfig;
    md: LayoutConfig;
    lg: LayoutConfig;
};

export const DEFAULT_LAYOUT_CONFIGS: LayoutConfigs = {
    sm: {
        cols: 1,
        rowHeight: 100,
        items: [
            { i: 'map', x: 0, y: 0, w: 1, h: 5, minH: 3, static: false },
            { i: 'solarsystem', x: 0, y: 5, w: 1, h: 4, minH: 2, static: false },
            { i: 'signatures', x: 0, y: 9, w: 1, h: 3, minH: 2, static: false },
            { i: 'audits', x: 0, y: 12, w: 1, h: 2, minH: 1, static: false },
            { i: 'ship-history', x: 0, y: 14, w: 1, h: 2, minH: 1, static: false },
            { i: 'characters', x: 0, y: 16, w: 1, h: 2, minH: 1, static: false },
            { i: 'killmails', x: 0, y: 18, w: 1, h: 3, minH: 2, static: false },
            { i: 'autopilot', x: 0, y: 21, w: 1, h: 4, minH: 2, static: false },
        ],
    },
    md: {
        cols: 2,
        rowHeight: 100,
        items: [
            { i: 'map', x: 0, y: 0, w: 2, h: 5, minH: 3, static: false },
            { i: 'solarsystem', x: 0, y: 5, w: 1, h: 4, minH: 2, static: false },
            { i: 'signatures', x: 0, y: 9, w: 1, h: 3, minH: 2, static: false },
            { i: 'audits', x: 0, y: 12, w: 1, h: 2, minH: 1, static: false },
            { i: 'ship-history', x: 0, y: 14, w: 1, h: 2, minH: 1, static: false },
            { i: 'characters', x: 1, y: 5, w: 1, h: 2, minH: 1, static: false },
            { i: 'killmails', x: 1, y: 7, w: 1, h: 3, minH: 2, static: false },
            { i: 'autopilot', x: 1, y: 10, w: 1, h: 4, minH: 2, static: false },
        ],
    },
    lg: {
        cols: 10,
        rowHeight: 100,
        items: [
            { i: 'map', x: 0, y: 0, w: 10, h: 5, minH: 3, static: false },
            { i: 'solarsystem', x: 0, y: 5, w: 4, h: 4, minH: 2, static: false },
            { i: 'signatures', x: 0, y: 9, w: 4, h: 3, minH: 2, static: false },
            { i: 'audits', x: 0, y: 12, w: 4, h: 2, minH: 1, static: false },
            { i: 'ship-history', x: 0, y: 14, w: 4, h: 2, minH: 1, static: false },
            { i: 'characters', x: 4, y: 5, w: 3, h: 2, minH: 1, static: false },
            { i: 'killmails', x: 4, y: 7, w: 3, h: 3, minH: 2, static: false },
            { i: 'autopilot', x: 7, y: 5, w: 3, h: 4, minH: 2, static: false },
        ],
    },
};

export const BREAKPOINT_WIDTHS = {
    sm: 640,
    md: 1024,
    lg: 1536,
};

export interface BreakpointConfig {
    key: Breakpoint;
    label: string;
    minWidth: number;
    description: string;
}

export const BREAKPOINT_OPTIONS: BreakpointConfig[] = [
    { key: 'sm', label: 'Small', minWidth: BREAKPOINT_WIDTHS.sm, description: 'Mobile & Tablet' },
    { key: 'md', label: 'Medium', minWidth: BREAKPOINT_WIDTHS.md, description: 'Tablet & Small Desktop' },
    { key: 'lg', label: 'Large', minWidth: BREAKPOINT_WIDTHS.lg, description: 'Desktop & Wide Screen' },
];

