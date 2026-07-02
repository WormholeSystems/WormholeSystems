import { scalePoint } from '../coords';
import type { EdgeGeometry, Vec2 } from '../types';

/**
 * Corner radius of the orthogonal elbow, in screen pixels. Paths are built in
 * scaled space so the radius stays constant across zoom levels, matching the
 * badge cluster and stroke widths which are constant-size too.
 */
export const CORNER_RADIUS = 10;

export function midpoint(from: Vec2, to: Vec2): Vec2 {
    return { x: (from.x + to.x) / 2, y: (from.y + to.y) / 2 };
}

/** The free-layout bezier, easing horizontally between the two endpoints. */
export function curvePath(from: Vec2, to: Vec2): string {
    const cp1x = from.x + (to.x - from.x) / 1.5;
    const cp2x = to.x - (to.x - from.x) / 1.5;
    return `M ${from.x} ${from.y} C ${cp1x} ${from.y}, ${cp2x} ${to.y}, ${to.x} ${to.y}`;
}

/**
 * The two right-angle turn points of an orthogonal elbow: the perpendicular run
 * sits at `bend` on the axis the connection exits (or the midpoint when unset).
 */
export function elbowCorners(geometry: Extract<EdgeGeometry, { kind: 'elbow' }>): [Vec2, Vec2] {
    const { from, to, fromNormal, bend } = geometry;
    if (fromNormal.x !== 0) {
        const midX = bend ?? (from.x + to.x) / 2;
        return [
            { x: midX, y: from.y },
            { x: midX, y: to.y },
        ];
    }
    const midY = bend ?? (from.y + to.y) / 2;
    return [
        { x: from.x, y: midY },
        { x: to.x, y: midY },
    ];
}

/** Polyline through `points` with each interior corner rounded by up to `radius`. */
export function roundedElbowPath(points: Vec2[], radius: number): string {
    const pts = points.filter((point, i) => i === 0 || Math.hypot(point.x - points[i - 1].x, point.y - points[i - 1].y) > 0.01);
    if (pts.length < 2) return '';
    let d = `M ${pts[0].x} ${pts[0].y}`;
    for (let i = 1; i < pts.length - 1; i++) {
        const prev = pts[i - 1];
        const curr = pts[i];
        const next = pts[i + 1];
        const lenIn = Math.hypot(curr.x - prev.x, curr.y - prev.y);
        const lenOut = Math.hypot(next.x - curr.x, next.y - curr.y);
        const r = Math.min(radius, lenIn / 2, lenOut / 2);
        const start = { x: curr.x + ((prev.x - curr.x) / lenIn) * r, y: curr.y + ((prev.y - curr.y) / lenIn) * r };
        const end = { x: curr.x + ((next.x - curr.x) / lenOut) * r, y: curr.y + ((next.y - curr.y) / lenOut) * r };
        d += ` L ${start.x} ${start.y} Q ${curr.x} ${curr.y} ${end.x} ${end.y}`;
    }
    const last = pts[pts.length - 1];
    d += ` L ${last.x} ${last.y}`;
    return d;
}

/**
 * The one place edge geometry crosses from base units into screen pixels:
 * scales the geometry, builds the SVG path, and returns the badge-cluster
 * centre alongside it.
 */
export function edgePathAndCenter(geometry: EdgeGeometry, scale: number): { d: string; center: Vec2 } {
    const from = scalePoint(geometry.from, scale);
    const to = scalePoint(geometry.to, scale);

    if (geometry.kind === 'curve') {
        return { d: curvePath(from, to), center: midpoint(from, to) };
    }

    const corners = elbowCorners({
        ...geometry,
        from,
        to,
        bend: geometry.bend === null ? null : geometry.bend * scale,
    });
    return {
        d: roundedElbowPath([from, corners[0], corners[1], to], CORNER_RADIUS),
        center: midpoint(corners[0], corners[1]),
    };
}
