// The chain laid out as a tree instead of dragged into shape. Positions are derived on the
// client and never stored, so manual placement finds its own untouched.

import type { MapSystemView } from '$lib/api/types/MapSystemView';
import { classMeta } from '$lib/map/classes';
import { systemName } from '$lib/map/system';

export interface TreeEdge {
	from: number;
	to: number;
}

export interface TreeInput {
	nodeIds: number[];
	edges: TreeEdge[];
	/** Pinned systems, which become the roots the branches grow out of. */
	rootIds: number[];
	/** A root like the pinned ones, and the one the column starts with. */
	homeId?: number | null;
	/** Used as the root when nothing is pinned and there is no home system. */
	fallbackRootId?: number | null;
	/** Orders siblings, and the separate trees, down the cross axis. */
	compareNodes?: (a: number, b: number) => number;
}

export interface TreeOptions {
	/** Distance between depth levels, in world units. */
	levelGap?: number;
	/** Smallest distance between siblings, in world units. */
	siblingGap?: number;
	/** Snaps the result onto the same grid the manual map uses. */
	gridSize?: number;
	marginX?: number;
	marginY?: number;
}

/** A node in the spanning forest, plus the scratch fields the walks mutate. */
interface LayoutNode {
	id: number;
	depth: number;
	parent: LayoutNode | null;
	children: LayoutNode[];
	/** 1-based position among siblings, so a shift can be spread across the subtrees between two contours. */
	siblingIndex: number;
	prelim: number;
	mod: number;
	change: number;
	shift: number;
	/** Stitches a shorter subtree's contour into its taller neighbour's. */
	thread: LayoutNode | null;
	/** Where a shift lands. Null until one is recorded, which reads as the node itself. */
	ancestor: LayoutNode | null;
	/** The final cross-axis coordinate, filled in by the second walk. */
	cross: number;
}

/**
 * A left-to-right spanning forest rooted at the pinned systems, everything else attaching
 * to whichever root reaches it first. Cross-axis packing is Reingold–Tilford in Buchheim's
 * linear form, so a tall branch pushes the next clear of its whole extent.
 *
 * Returns a node top-left per system id, in world units, snapped to the grid.
 */
export function computeTreeLayout(
	input: TreeInput,
	options: TreeOptions = {},
): Map<number, { x: number; y: number }> {
	const gridSize = options.gridSize ?? 20;
	const snap = (value: number) => Math.round(value / gridSize) * gridSize;
	// Down the cross axis, half a cell. A parent sits at the midpoint of its first and last
	// child, and with an even number of children that is half a sibling pitch off one of
	// them. Snapping that to whole cells needs a pitch of two cells or more, which is a
	// 40px gap between neighbours; at half a cell a pitch of one cell is enough, so the
	// nodes can sit as close as the row under them allows and the parent still lands level
	// between them rather than a snap to one side.
	const crossStep = gridSize / 2;
	const snapCross = (value: number) => Math.round(value / crossStep) * crossStep;
	// Wide enough for an edge's badge cluster at its midpoint to clear both nodes.
	const levelGap = snap(options.levelGap ?? 320);
	// A whole number of cells, so half of it is a whole number of cross steps and every
	// midpoint the layout can produce is a position it can actually place. Node height is
	// two cells, so this leaves a gap of one cell between neighbours.
	const siblingGap = Math.max(gridSize, snap(options.siblingGap ?? 60));
	// Where the first node's top-left sits.
	const marginX = snap(options.marginX ?? 20);
	const marginY = snap(options.marginY ?? 20);

	// --- The undirected graph of systems. ---
	const adjacency = new Map<number, number[]>();
	for (const id of input.nodeIds) adjacency.set(id, []);
	for (const edge of input.edges) {
		if (edge.from === edge.to || !adjacency.has(edge.from) || !adjacency.has(edge.to)) continue;
		adjacency.get(edge.from)!.push(edge.to);
		adjacency.get(edge.to)!.push(edge.from);
	}
	for (const [id, neighbours] of adjacency) adjacency.set(id, [...new Set(neighbours)]);

	// --- Carve a spanning forest out of it, breadth-first. ---
	const depthOf = new Map<number, number>();
	const childrenOf = new Map<number, number[]>();
	for (const id of input.nodeIds) childrenOf.set(id, []);
	const visited = new Set<number>();
	const roots: number[] = [];
	const queue: number[] = [];

	const drain = () => {
		while (queue.length > 0) {
			const current = queue.shift()!;
			for (const neighbour of adjacency.get(current) ?? []) {
				if (visited.has(neighbour)) continue;
				visited.add(neighbour);
				depthOf.set(neighbour, depthOf.get(current)! + 1);
				childrenOf.get(current)!.push(neighbour);
				queue.push(neighbour);
			}
		}
	};
	const addRoot = (id: number) => {
		roots.push(id);
		depthOf.set(id, 0);
		visited.add(id);
		queue.push(id);
	};

	// Pinned first, stranded after, so the left column reads in order of what matters.
	const chosen = new Set<number>();
	const home = input.homeId != null && adjacency.has(input.homeId) ? input.homeId : null;
	const candidates = [
		...(home === null ? [] : [home]),
		...input.rootIds.filter((id) => adjacency.has(id) && id !== home),
	];
	if (
		candidates.length === 0 &&
		input.fallbackRootId != null &&
		adjacency.has(input.fallbackRootId)
	) {
		candidates.push(input.fallbackRootId);
	}
	// All roots seeded before the walk, so each system attaches to the nearest one.
	for (const root of candidates) {
		if (visited.has(root)) continue;
		chosen.add(root);
		addRoot(root);
	}
	drain();

	// Whatever the roots could not reach becomes a tree of its own, densest first.
	const stranded = input.nodeIds
		.filter((id) => !visited.has(id))
		.sort((a, b) => adjacency.get(b)!.length - adjacency.get(a)!.length || a - b);
	for (const root of stranded) {
		if (visited.has(root)) continue;
		addRoot(root);
		drain();
	}

	const compare = input.compareNodes;
	if (compare) {
		for (const [, children] of childrenOf) children.sort(compare);
	}
	// Home, then pinned, then unreached; the comparator only orders within each rank.
	const rank = (id: number) => (id === home ? 0 : chosen.has(id) ? 1 : 2);
	roots.sort((a, b) => rank(a) - rank(b) || (compare ? compare(a, b) : 0));

	// --- The forest as linked records the walks can chew on. ---
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
			ancestor: null,
			cross: 0,
		};
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

	// The node continuing the left / right contour: the first / last child, or, for a
	// leaf, the thread into a neighbour's contour.
	const nextLeft = (node: LayoutNode) => node.children[0] ?? node.thread;
	const nextRight = (node: LayoutNode) => node.children[node.children.length - 1] ?? node.thread;
	const leftSiblingOf = (node: LayoutNode) =>
		node.parent && node.siblingIndex > 1 ? node.parent.children[node.siblingIndex - 2] : null;

	const moveSubtree = (left: LayoutNode, right: LayoutNode, distance: number) => {
		const subtrees = right.siblingIndex - left.siblingIndex;
		right.change -= distance / subtrees;
		right.shift += distance;
		left.change += distance / subtrees;
		right.prelim += distance;
		right.mod += distance;
	};

	/** The recorded ancestor, or the node itself when none has been recorded yet. */
	const ancestorOf = (node: LayoutNode) => node.ancestor ?? node;

	// Where a shift is absorbed: the recorded ancestor when it is a sibling of `node`,
	// else the running default.
	const ancestorFor = (inner: LayoutNode, node: LayoutNode, fallback: LayoutNode) =>
		ancestorOf(inner).parent === node.parent ? ancestorOf(inner) : fallback;

	const executeShifts = (node: LayoutNode) => {
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

	// Slide a subtree along until its left contour clears its left siblings' right
	// contour by the sibling gap, threading the shorter side so deeper levels stay apart.
	const apportion = (node: LayoutNode, defaultAncestor: LayoutNode): LayoutNode => {
		const leftSibling = leftSiblingOf(node);
		if (!leftSibling) return defaultAncestor;

		let innerRight = node;
		let outerRight = node;
		let innerLeft = leftSibling;
		let outerLeft = node.parent!.children[0];
		let sInnerRight = innerRight.mod;
		let sOuterRight = outerRight.mod;
		let sInnerLeft = innerLeft.mod;
		let sOuterLeft = outerLeft.mod;

		while (nextRight(innerLeft) && nextLeft(innerRight)) {
			innerLeft = nextRight(innerLeft)!;
			innerRight = nextLeft(innerRight)!;
			outerLeft = nextLeft(outerLeft)!;
			outerRight = nextRight(outerRight)!;
			outerRight.ancestor = node;
			const shift = innerLeft.prelim + sInnerLeft - (innerRight.prelim + sInnerRight) + siblingGap;
			if (shift > 0) {
				moveSubtree(ancestorFor(innerLeft, node, defaultAncestor), node, shift);
				sInnerRight += shift;
				sOuterRight += shift;
			}
			sInnerLeft += innerLeft.mod;
			sInnerRight += innerRight.mod;
			sOuterLeft += outerLeft.mod;
			sOuterRight += outerRight.mod;
		}
		if (nextRight(innerLeft) && !nextRight(outerRight)) {
			outerRight.thread = nextRight(innerLeft);
			outerRight.mod += sInnerLeft - sOuterRight;
		} else if (nextLeft(innerRight) && !nextLeft(outerLeft)) {
			outerLeft.thread = nextLeft(innerRight);
			outerLeft.mod += sInnerRight - sOuterLeft;
			defaultAncestor = node;
		}
		return defaultAncestor;
	};

	// First walk: a preliminary cross position per subtree, relative to its parent,
	// resolving sibling overlaps on the way up.
	const firstWalk = (node: LayoutNode) => {
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

	// Second walk: sum the modifiers down each path, turning preliminary positions absolute.
	const secondWalk = (node: LayoutNode, modSum: number) => {
		node.cross = node.prelim + modSum;
		for (const child of node.children) secondWalk(child, modSum + node.mod);
	};

	// A virtual super-root contour-packs the trees like any other siblings, so a shallow
	// tree rises into a deeper neighbour's empty rows. Never rendered.
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
		ancestor: null,
		cross: 0,
	};
	superRoot.children.forEach((child, index) => {
		child.parent = superRoot;
		child.siblingIndex = index + 1;
	});
	firstWalk(superRoot);
	secondWalk(superRoot, 0);

	// The walk centres the forest on zero, so drop the whole thing onto the top margin.
	let minCross = Infinity;
	for (const node of nodes.values()) minCross = Math.min(minCross, node.cross);
	if (!Number.isFinite(minCross)) minCross = 0;

	const positions = new Map<number, { x: number; y: number }>();
	for (const node of nodes.values()) {
		positions.set(node.id, {
			x: snap(marginX + node.depth * levelGap),
			y: snapCross(marginY + node.cross - minCross),
		});
	}
	return positions;
}

/**
 * Where a system's class sits among its siblings: high, low and null sec, Pochven, then
 * wormhole space by class, and anything unknown (a ghost) last.
 */
function classRank(system: MapSystemView): number {
	if (system.kind === 'ghost') return 1000;
	return classMeta(system.wormhole_class_id, system.security_status).sortWeight;
}

function regionOf(system: MapSystemView): string {
	return system.kind === 'system' ? system.region : '';
}

/**
 * The legacy sibling order: systems without an alias first, aliased ones by alias, then
 * class, region and name, so a branch reads the same way every time.
 */
export function compareForTree(systems: Map<number, MapSystemView>) {
	return (a: number, b: number): number => {
		const left = systems.get(a);
		const right = systems.get(b);
		if (!left || !right) return a - b;
		if (left.alias && !right.alias) return 1;
		if (!left.alias && right.alias) return -1;
		const byAlias = (left.alias ?? '').localeCompare(right.alias ?? '');
		if (byAlias !== 0) return byAlias;
		const byClass = classRank(left) - classRank(right);
		if (byClass !== 0) return byClass;
		const byRegion = regionOf(left).localeCompare(regionOf(right));
		if (byRegion !== 0) return byRegion;
		return (systemName(left) ?? '').localeCompare(systemName(right) ?? '') || a - b;
	};
}
