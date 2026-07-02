import type { EdgeGeometry, EdgeInput, Rect, Vec2 } from '../types';

/** Spacing between connections that leave the same node edge, in base units. */
export const PARALLEL_SPACING = 14;
/** Spacing between the perpendicular runs of those connections' bends, in base units. */
export const BEND_SPACING = 16;

export type Endpoints = { from: Vec2; to: Vec2; fromNormal: Vec2; toNormal: Vec2 };

/**
 * Connects the centre of each box's facing edge, leaving perpendicular to it.
 *
 * Prefer the left/right edges whenever the boxes are separated horizontally — then the
 * smoothstep drops its long vertical run through the clear lane between the columns
 * (down, then right into the node) instead of routing down-over-down with the second
 * vertical run cutting straight through the column of stacked siblings. Top/bottom
 * edges are only used when the boxes share a column, so there is no horizontal lane.
 */
export function edgeCenterConnection(source: Rect, target: Rect): Endpoints {
    const dx = target.centerX - source.centerX;
    const dy = target.centerY - source.centerY;

    const separatedX = target.minX > source.maxX || source.minX > target.maxX;
    const separatedY = target.minY > source.maxY || source.minY > target.maxY;
    const useHorizontal = separatedX || (!separatedY && Math.abs(dx) >= Math.abs(dy));

    if (useHorizontal) {
        const rightward = dx >= 0;
        return {
            from: { x: rightward ? source.maxX : source.minX, y: source.centerY },
            to: { x: rightward ? target.minX : target.maxX, y: target.centerY },
            fromNormal: { x: rightward ? 1 : -1, y: 0 },
            toNormal: { x: rightward ? -1 : 1, y: 0 },
        };
    }

    const downward = dy >= 0;
    return {
        from: { x: source.centerX, y: downward ? source.maxY : source.minY },
        to: { x: target.centerX, y: downward ? target.minY : target.maxY },
        fromNormal: { x: 0, y: downward ? 1 : -1 },
        toNormal: { x: 0, y: downward ? -1 : 1 },
    };
}

type Port = { endpoint: Vec2; normal: Vec2; box: Rect; sortKey: number };

/**
 * Fans out connections that share a node edge so parallel lines don't overlap:
 * each shared edge spreads its endpoints along itself, ordered by the other end's
 * position so the lines stay untangled.
 */
function spreadSharedEdges(ports: Port[]): void {
    if (ports.length < 2) return;
    const alongY = ports[0].normal.x !== 0;
    const box = ports[0].box;
    const extent = alongY ? box.maxY - box.minY : box.maxX - box.minX;
    const spacing = Math.min(PARALLEL_SPACING, (extent * 0.7) / (ports.length - 1));
    ports.sort((a, b) => a.sortKey - b.sortKey);
    ports.forEach((port, i) => {
        const offset = (i - (ports.length - 1) / 2) * spacing;
        if (alongY) {
            port.endpoint.y += offset;
        } else {
            port.endpoint.x += offset;
        }
    });
}

type RoutedEdge = Extract<EdgeGeometry, { kind: 'elbow' }> & {
    sourceBox: Rect;
    targetBox: Rect;
    /** Perpendicular distance and signed offset of the far end, for fan ordering. */
    distance: number;
    signed: number;
};

/**
 * Tree-layout routing, a global pass over all edges: connects facing box edges,
 * fans out endpoints that share a node edge, then staggers the perpendicular runs
 * of connections fanning out from the same node so they nest instead of stacking.
 * Edges whose nodes are not both measured fall back to a straight curve between
 * their anchors (edges missing an anchor entirely are dropped).
 */
export function computeTreeEdgeGeometries(
    edges: EdgeInput[],
    rects: ReadonlyMap<number, Rect>,
    anchors: ReadonlyMap<number, Vec2>,
): Map<number, EdgeGeometry> {
    const geometries = new Map<number, EdgeGeometry>();
    const routed: RoutedEdge[] = [];

    for (const edge of edges) {
        const sourceBox = rects.get(edge.sourceId);
        const targetBox = rects.get(edge.targetId);
        if (!sourceBox || !targetBox) {
            const from = anchors.get(edge.sourceId);
            const to = anchors.get(edge.targetId);
            if (from && to) {
                geometries.set(edge.id, { id: edge.id, kind: 'curve', from, to });
            }
            continue;
        }
        const ends = edgeCenterConnection(sourceBox, targetBox);
        const item: RoutedEdge = {
            id: edge.id,
            kind: 'elbow',
            from: { ...ends.from },
            to: { ...ends.to },
            fromNormal: ends.fromNormal,
            toNormal: ends.toNormal,
            bend: null,
            sourceBox,
            targetBox,
            distance: 0,
            signed: 0,
        };
        routed.push(item);
        geometries.set(edge.id, item);
    }

    // Fan out the endpoints that share a node edge so parallel lines don't overlap.
    const sharedEdges = new Map<string, Port[]>();
    const register = (endpoint: Vec2, normal: Vec2, box: Rect, other: Rect): void => {
        const port: Port = { endpoint, normal, box, sortKey: normal.x !== 0 ? other.centerY : other.centerX };
        const key = `${box.centerX},${box.centerY}|${normal.x},${normal.y}`;
        (sharedEdges.get(key) ?? sharedEdges.set(key, []).get(key)!).push(port);
    };
    for (const item of routed) {
        register(item.from, item.fromNormal, item.sourceBox, item.targetBox);
        register(item.to, item.toNormal, item.targetBox, item.sourceBox);
    }
    for (const ports of sharedEdges.values()) {
        spreadSharedEdges(ports);
    }

    // Stagger the perpendicular run of connections that fan out from the same node so
    // they nest instead of stacking. Group by the node the fan leaves on its primary
    // axis (the left node for horizontal links, the top node for vertical), regardless
    // of which end is the stored source, and order each group by how far its far end
    // sits so the runs don't cross.
    const fans = new Map<string, RoutedEdge[]>();
    for (const item of routed) {
        const horizontal = item.fromNormal.x !== 0;
        const sourceFirst = horizontal ? item.sourceBox.centerX <= item.targetBox.centerX : item.sourceBox.centerY <= item.targetBox.centerY;
        const primary = sourceFirst ? item.sourceBox : item.targetBox;
        const other = sourceFirst ? item.targetBox : item.sourceBox;
        const along = horizontal ? other.centerY - primary.centerY : other.centerX - primary.centerX;
        item.distance = Math.abs(along);
        item.signed = along;
        const key = `${horizontal ? 'h' : 'v'}|${primary.centerX},${primary.centerY}`;
        (fans.get(key) ?? fans.set(key, []).get(key)!).push(item);
    }
    for (const group of fans.values()) {
        if (group.length < 2) continue;
        const horizontal = group[0].fromNormal.x !== 0;
        // Keep the whole fan inside the lane between the two columns: when there are more
        // connections than BEND_SPACING would fit in the gap, shrink the spacing so the
        // outermost runs still clear the neighbouring nodes instead of cutting through them.
        const gap = Math.min(...group.map((item) => Math.abs(horizontal ? item.to.x - item.from.x : item.to.y - item.from.y)));
        const spacing = Math.min(BEND_SPACING, (gap * 0.8) / (group.length - 1));
        // Farthest target bends closest to the node; ties (above vs below) split by side.
        group.sort((a, b) => b.distance - a.distance || a.signed - b.signed);
        group.forEach((item, i) => {
            const base = horizontal ? (item.from.x + item.to.x) / 2 : (item.from.y + item.to.y) / 2;
            item.bend = base + (i - (group.length - 1) / 2) * spacing;
        });
    }

    return geometries;
}
