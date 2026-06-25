import type { Coordinates } from '../types';

export type TreeLayoutEdge = {
    from: number;
    to: number;
};

export type TreeLayoutInput = {
    nodeIds: number[];
    edges: TreeLayoutEdge[];
    /** Pinned systems that should become the roots branches grow out of. */
    rootIds: number[];
    /** Used as a root when no systems are pinned (e.g. the map's home system). */
    fallbackRootId?: number | null;
    /** Orders siblings (and separate trees) along the cross axis. */
    compareNodes?: (a: number, b: number) => number;
};

export type TreeLayoutOptions = {
    /** Distance between depth levels (x), in base units. */
    levelGap?: number;
    /** Minimum distance between siblings (y), in base units. */
    siblingGap?: number;
    /** Snaps positions onto the same grid the manual map uses. */
    gridSize?: number;
    /** Padding kept around the laid-out content, in base units. */
    margin?: number;
};

/**
 * Lays systems out as a left-to-right spanning forest rooted at the pinned systems.
 *
 * Each pinned system starts its own tree; every other system attaches to the nearest
 * root via breadth-first search. Returns a centre position per system id, in base
 * (unscaled) units snapped to the grid. Loop-back connections need no special handling
 * here — the renderer simply draws every connection between the computed positions.
 */
export function computeTreeLayout(input: TreeLayoutInput, options: TreeLayoutOptions = {}): Map<number, Coordinates> {
    const gridSize = options.gridSize ?? 20;
    // Snap the spacings to whole grid cells. Otherwise a gap that is not a multiple
    // of the grid (e.g. 70 on a 20 grid) makes per-node snapping alternate between
    // one and two cells, so sibling spacing looks irregular.
    const snap = (value: number): number => Math.round(value / gridSize) * gridSize;
    // levelGap is wide enough that a connection's midpoint (where the mass /
    // ship-size badges sit) clears the fixed-width nodes, which paint over the SVG.
    const levelGap = snap(options.levelGap ?? 320);
    // Sized so plain 40px nodes keep a small gap and 60px pilot rows still don't
    // overlap; tighter than this starts overlapping systems that have online pilots.
    const siblingGap = snap(options.siblingGap ?? 60);
    const margin = snap(options.margin ?? 80);

    const adjacency = new Map<number, number[]>();
    for (const id of input.nodeIds) {
        adjacency.set(id, []);
    }
    for (const edge of input.edges) {
        if (edge.from === edge.to) {
            continue;
        }
        if (!adjacency.has(edge.from) || !adjacency.has(edge.to)) {
            continue;
        }
        adjacency.get(edge.from)!.push(edge.to);
        adjacency.get(edge.to)!.push(edge.from);
    }
    for (const [id, neighbours] of adjacency) {
        adjacency.set(id, [...new Set(neighbours)]);
    }

    const depthOf = new Map<number, number>();
    const childrenOf = new Map<number, number[]>();
    const parentOf = new Map<number, number>();
    for (const id of input.nodeIds) {
        childrenOf.set(id, []);
    }
    const visited = new Set<number>();
    const roots: number[] = [];

    const queue: number[] = [];
    const processQueue = (): void => {
        while (queue.length > 0) {
            const current = queue.shift()!;
            for (const neighbour of adjacency.get(current) ?? []) {
                if (visited.has(neighbour)) {
                    continue;
                }
                visited.add(neighbour);
                depthOf.set(neighbour, (depthOf.get(current) ?? 0) + 1);
                childrenOf.get(current)!.push(neighbour);
                parentOf.set(neighbour, current);
                queue.push(neighbour);
            }
        }
    };

    const addRoot = (id: number): void => {
        roots.push(id);
        depthOf.set(id, 0);
        visited.add(id);
        queue.push(id);
    };

    const candidateRoots = input.rootIds.filter((id) => adjacency.has(id));
    if (candidateRoots.length === 0 && input.fallbackRootId != null && adjacency.has(input.fallbackRootId)) {
        candidateRoots.push(input.fallbackRootId);
    }
    // Seed every pinned root first so they share one breadth-first front and each
    // unpinned system attaches to whichever root is closest.
    for (const root of candidateRoots) {
        if (!visited.has(root)) {
            addRoot(root);
        }
    }
    processQueue();

    // Whatever the roots could not reach (disconnected systems) becomes its own
    // tree, densest component first, parked beside the rooted forest.
    const remaining = input.nodeIds.filter((id) => !visited.has(id)).sort((a, b) => adjacency.get(b)!.length - adjacency.get(a)!.length || a - b);
    for (const root of remaining) {
        if (visited.has(root)) {
            continue;
        }
        addRoot(root);
        processQueue();
    }

    // Order siblings (and the separate trees) so branches read consistently.
    if (input.compareNodes) {
        const compare = input.compareNodes;
        for (const [, children] of childrenOf) {
            children.sort(compare);
        }
        roots.sort(compare);
    }

    // Group nodes by depth, ordered by a pre-order walk so each subtree stays contiguous
    // (which keeps branches from crossing once they are pulled together below).
    const levels: number[][] = [];
    const collectLevels = (node: number): void => {
        const depth = depthOf.get(node)!;
        (levels[depth] ??= []).push(node);
        for (const child of childrenOf.get(node)!) {
            collectLevels(child);
        }
    };
    for (const root of roots) {
        collectLevels(root);
    }

    // Cross-axis (y) coordinate per node, seeded uniformly within each level.
    const y = new Map<number, number>();
    for (const level of levels) {
        level.forEach((node, index) => y.set(node, index * siblingGap));
    }

    // Place a level's nodes as close to their desired y as possible while keeping at least
    // siblingGap between neighbours and their order intact (isotonic regression via PAV).
    // This is what lets the height follow the widest level: a sparse deeper level packs
    // tightly around its parent, and only spreads the level above once it outgrows it.
    const separateLevel = (level: number[]): void => {
        const blocks: { value: number; weight: number }[] = [];
        level.forEach((node, index) => {
            let value = y.get(node)! - index * siblingGap;
            let weight = 1;
            while (blocks.length > 0 && blocks[blocks.length - 1].value > value) {
                const previous = blocks.pop()!;
                value = (previous.value * previous.weight + value * weight) / (previous.weight + weight);
                weight += previous.weight;
            }
            blocks.push({ value, weight });
        });
        let index = 0;
        for (const block of blocks) {
            for (let offset = 0; offset < block.weight; offset++) {
                y.set(level[index], block.value + index * siblingGap);
                index += 1;
            }
        }
    };

    // Alternate pulling children under their parent and parents to their children's centre;
    // a handful of passes converges for a tree.
    const PASSES = 6;
    for (let pass = 0; pass < PASSES; pass++) {
        for (let depth = 1; depth < levels.length; depth++) {
            for (const node of levels[depth]) {
                const parent = parentOf.get(node);
                if (parent !== undefined) {
                    y.set(node, y.get(parent)!);
                }
            }
            separateLevel(levels[depth]);
        }
        for (let depth = levels.length - 2; depth >= 0; depth--) {
            for (const node of levels[depth]) {
                const children = childrenOf.get(node)!;
                if (children.length > 0) {
                    y.set(node, children.reduce((total, child) => total + y.get(child)!, 0) / children.length);
                }
            }
            separateLevel(levels[depth]);
        }
    }

    let minY = Infinity;
    for (const value of y.values()) {
        minY = Math.min(minY, value);
    }
    if (!Number.isFinite(minY)) {
        minY = 0;
    }

    const positions = new Map<number, Coordinates>();
    for (const id of input.nodeIds) {
        const depth = depthOf.get(id);
        const yValue = y.get(id);
        if (depth === undefined || yValue === undefined) {
            continue;
        }
        positions.set(id, { x: snap(margin + depth * levelGap), y: snap(margin + yValue - minY) });
    }

    return positions;
}
