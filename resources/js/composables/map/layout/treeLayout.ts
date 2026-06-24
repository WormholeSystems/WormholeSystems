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
    /** Distance between siblings (y), in base units. */
    siblingGap?: number;
    /** Empty sibling slots inserted between separate trees of the forest. */
    treeGap?: number;
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
    const treeGap = options.treeGap ?? 2;

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

    // First walk: give every leaf its own cross-axis slot, every parent the midpoint
    // of its children. A shared slot counter across roots spreads the trees apart.
    const slot = new Map<number, number>();
    let nextLeafSlot = 0;
    const assignSlots = (node: number): void => {
        const children = childrenOf.get(node)!;
        if (children.length === 0) {
            slot.set(node, nextLeafSlot);
            nextLeafSlot += 1;
            return;
        }
        for (const child of children) {
            assignSlots(child);
        }
        const first = slot.get(children[0])!;
        const last = slot.get(children[children.length - 1])!;
        slot.set(node, (first + last) / 2);
    };
    for (const root of roots) {
        assignSlots(root);
        nextLeafSlot += treeGap;
    }

    const positions = new Map<number, Coordinates>();
    for (const id of input.nodeIds) {
        const depth = depthOf.get(id);
        const slotValue = slot.get(id);
        if (depth === undefined || slotValue === undefined) {
            continue;
        }
        positions.set(id, { x: snap(margin + depth * levelGap), y: snap(margin + slotValue * siblingGap) });
    }

    return positions;
}
