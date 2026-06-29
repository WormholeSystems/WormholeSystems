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
    /** Left padding before the first column, in base units. */
    marginX?: number;
    /** Top padding above the first row, in base units. */
    marginY?: number;
};

/**
 * A forest node, linked to its parent and children, plus the scratch fields the
 * Reingold–Tilford layout mutates. Built once the spanning forest is known.
 */
type LayoutNode = {
    id: number;
    depth: number;
    parent: LayoutNode | null;
    children: LayoutNode[];
    /** 1-based position among siblings; lets moveSubtree spread a shift across the subtrees between two contours. */
    siblingIndex: number;
    prelim: number;
    mod: number;
    change: number;
    shift: number;
    /** Stitches a shorter subtree's contour into its taller neighbour's. */
    thread: LayoutNode | null;
    ancestor: LayoutNode;
    /** Final cross-axis (y) coordinate, filled by secondWalk. */
    cross: number;
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
    // A little left padding, and a top padding one system height shorter than the left so
    // the first row sits close to the top edge.
    const marginX = snap(options.marginX ?? 60);
    const marginY = snap(options.marginY ?? 40);

    // --- Build the undirected graph of systems. ---
    const adjacency = new Map<number, number[]>();
    for (const id of input.nodeIds) {
        adjacency.set(id, []);
    }
    for (const edge of input.edges) {
        if (edge.from === edge.to || !adjacency.has(edge.from) || !adjacency.has(edge.to)) {
            continue;
        }
        adjacency.get(edge.from)!.push(edge.to);
        adjacency.get(edge.to)!.push(edge.from);
    }
    for (const [id, neighbours] of adjacency) {
        adjacency.set(id, [...new Set(neighbours)]);
    }

    // --- Carve a spanning forest out of the graph via breadth-first search. ---
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
                depthOf.set(neighbour, depthOf.get(current)! + 1);
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

    // --- Materialise the forest as linked node records for the layout passes. ---
    const nodes = new Map<number, LayoutNode>();
    for (const id of input.nodeIds) {
        const node: LayoutNode = {
            id,
            depth: depthOf.get(id) ?? 0,
            parent: null,
            children: [],
            siblingIndex: 1,
            prelim: 0,
            mod: 0,
            change: 0,
            shift: 0,
            thread: null,
            ancestor: null as unknown as LayoutNode,
            cross: 0,
        };
        node.ancestor = node;
        nodes.set(id, node);
    }
    for (const [id, childIds] of childrenOf) {
        const node = nodes.get(id)!;
        node.children = childIds.map((childId) => nodes.get(childId)!);
        node.children.forEach((child, index) => {
            child.parent = node;
            child.siblingIndex = index + 1;
        });
    }

    // --- Cross-axis (y) placement: Reingold–Tilford, Buchheim et al.'s linear formulation. ---
    // Each subtree is laid out against its siblings' contours, so a tall subtree pushes the next
    // sibling clear of its whole extent — not just one row. This keeps a node's siblings (e.g.
    // other pinned roots) out of the band its own children occupy, while sparse subtrees still
    // nest tightly. siblingGap is the minimum gap kept between adjacent nodes on any level.

    // The node continuing the left / right contour: the first / last child, or — for a leaf — the
    // threaded node into a neighbour's contour.
    const nextLeft = (node: LayoutNode): LayoutNode | null => node.children[0] ?? node.thread;
    const nextRight = (node: LayoutNode): LayoutNode | null => node.children[node.children.length - 1] ?? node.thread;
    const leftSiblingOf = (node: LayoutNode): LayoutNode | null =>
        node.parent && node.siblingIndex > 1 ? node.parent.children[node.siblingIndex - 2] : null;

    const moveSubtree = (left: LayoutNode, right: LayoutNode, distance: number): void => {
        const subtrees = right.siblingIndex - left.siblingIndex;
        right.change -= distance / subtrees;
        right.shift += distance;
        left.change += distance / subtrees;
        right.prelim += distance;
        right.mod += distance;
    };

    // Where a shift is absorbed: vInnerMinus's recorded ancestor if it is a sibling of node (a
    // parent-pointer compare), else the running default ancestor.
    const ancestorFor = (vInnerMinus: LayoutNode, node: LayoutNode, defaultAncestor: LayoutNode): LayoutNode =>
        vInnerMinus.ancestor.parent === node.parent ? vInnerMinus.ancestor : defaultAncestor;

    const executeShifts = (node: LayoutNode): void => {
        let shift = 0;
        let change = 0;
        for (let index = node.children.length - 1; index >= 0; index--) {
            const child = node.children[index];
            child.prelim += shift;
            child.mod += shift;
            change += child.change;
            shift += child.shift + change;
        }
    };

    // Slide node's subtree right until its left contour clears its left siblings' right contour by
    // siblingGap, threading the shorter side so deeper levels stay separated too.
    const apportion = (node: LayoutNode, defaultAncestor: LayoutNode): LayoutNode => {
        const leftSibling = leftSiblingOf(node);
        if (!leftSibling) {
            return defaultAncestor;
        }
        let vInnerPlus = node;
        let vOuterPlus = node;
        let vInnerMinus = leftSibling;
        let vOuterMinus = node.parent!.children[0];
        let sInnerPlus = vInnerPlus.mod;
        let sOuterPlus = vOuterPlus.mod;
        let sInnerMinus = vInnerMinus.mod;
        let sOuterMinus = vOuterMinus.mod;
        while (nextRight(vInnerMinus) && nextLeft(vInnerPlus)) {
            vInnerMinus = nextRight(vInnerMinus)!;
            vInnerPlus = nextLeft(vInnerPlus)!;
            vOuterMinus = nextLeft(vOuterMinus)!;
            vOuterPlus = nextRight(vOuterPlus)!;
            vOuterPlus.ancestor = node;
            const shift = vInnerMinus.prelim + sInnerMinus - (vInnerPlus.prelim + sInnerPlus) + siblingGap;
            if (shift > 0) {
                moveSubtree(ancestorFor(vInnerMinus, node, defaultAncestor), node, shift);
                sInnerPlus += shift;
                sOuterPlus += shift;
            }
            sInnerMinus += vInnerMinus.mod;
            sInnerPlus += vInnerPlus.mod;
            sOuterMinus += vOuterMinus.mod;
            sOuterPlus += vOuterPlus.mod;
        }
        if (nextRight(vInnerMinus) && !nextRight(vOuterPlus)) {
            vOuterPlus.thread = nextRight(vInnerMinus);
            vOuterPlus.mod += sInnerMinus - sOuterPlus;
        } else if (nextLeft(vInnerPlus) && !nextLeft(vOuterMinus)) {
            vOuterMinus.thread = nextLeft(vInnerPlus);
            vOuterMinus.mod += sInnerPlus - sOuterMinus;
            defaultAncestor = node;
        }
        return defaultAncestor;
    };

    // First pass: give each subtree a preliminary x relative to its parent, resolving overlaps
    // between sibling subtrees as it goes.
    const firstWalk = (node: LayoutNode): void => {
        if (node.children.length === 0) {
            const leftSibling = leftSiblingOf(node);
            node.prelim = leftSibling ? leftSibling.prelim + siblingGap : 0;
            return;
        }
        let defaultAncestor = node.children[0];
        for (const child of node.children) {
            firstWalk(child);
            defaultAncestor = apportion(child, defaultAncestor);
        }
        executeShifts(node);
        const midpoint = (node.children[0].prelim + node.children[node.children.length - 1].prelim) / 2;
        const leftSibling = leftSiblingOf(node);
        if (leftSibling) {
            node.prelim = leftSibling.prelim + siblingGap;
            node.mod = node.prelim - midpoint;
        } else {
            node.prelim = midpoint;
        }
    };

    // Second pass: sum the modifiers down each path to turn preliminary positions into absolute ones.
    const secondWalk = (node: LayoutNode, modSum: number): void => {
        node.cross = node.prelim + modSum;
        for (const child of node.children) {
            secondWalk(child, modSum + node.mod);
        }
    };

    // Hang every root off one virtual super-root and lay out the whole forest in a single pass, so
    // the roots are contour-packed exactly like any other siblings: a shallow tree rises into a
    // deeper neighbour's empty columns instead of being parked below its lowest node. The
    // super-root is never rendered — it just gives the roots a common parent for the two walks.
    const superRoot: LayoutNode = {
        id: -1,
        depth: -1,
        parent: null,
        children: roots.map((id) => nodes.get(id)!),
        siblingIndex: 1,
        prelim: 0,
        mod: 0,
        change: 0,
        shift: 0,
        thread: null,
        ancestor: null as unknown as LayoutNode,
        cross: 0,
    };
    superRoot.ancestor = superRoot;
    superRoot.children.forEach((child, index) => {
        child.parent = superRoot;
        child.siblingIndex = index + 1;
    });
    firstWalk(superRoot);
    secondWalk(superRoot, 0);

    // firstWalk centres the forest around zero, so cross can be negative; drop it onto the top margin.
    let minCross = Infinity;
    for (const node of nodes.values()) {
        minCross = Math.min(minCross, node.cross);
    }
    if (!Number.isFinite(minCross)) {
        minCross = 0;
    }

    const positions = new Map<number, Coordinates>();
    for (const node of nodes.values()) {
        positions.set(node.id, {
            x: snap(marginX + node.depth * levelGap),
            y: snap(marginY + node.cross - minCross),
        });
    }

    return positions;
}
