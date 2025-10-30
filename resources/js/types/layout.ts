/**
 * Layout item representing a single component in the grid
 */
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

/**
 * Layout configuration for a specific breakpoint
 */
export interface LayoutConfig {
    cols: number; // Number of columns
    rowHeight: number; // Height of each row in pixels
    items: LayoutItem[];
}

/**
 * Complete breakpoint definition including layout and metadata
 */
export interface BreakpointDefinition {
    key: string; // Unique identifier
    label: string; // Display name
    minWidth: number; // Minimum screen width in pixels
    description?: string; // Optional description
    cols: number; // Number of grid columns
    rowHeight: number; // Height of each row in pixels
    items: LayoutItem[]; // Layout items/components
}

/**
 * Collection of breakpoint definitions indexed by key
 */
export type BreakpointsConfig = Record<string, BreakpointDefinition>;
