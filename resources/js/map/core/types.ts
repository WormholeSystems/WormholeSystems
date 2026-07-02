export type Vec2 = { x: number; y: number };

export type Size = { width: number; height: number };

/** An axis-aligned node bounding box with its centre precomputed. */
export type Rect = {
    minX: number;
    minY: number;
    maxX: number;
    maxY: number;
    centerX: number;
    centerY: number;
};

/** A connection reduced to what edge routing needs. */
export type EdgeInput = {
    id: number;
    sourceId: number;
    targetId: number;
};

/**
 * Routed edge geometry in base units. `curve` is the free-layout bezier (also the
 * fallback while nodes are unmeasured); `elbow` is the tree layout's orthogonal
 * smoothstep, leaving each node perpendicular to `fromNormal`/`toNormal` with its
 * perpendicular run at `bend` (or the midpoint when null).
 */
export type EdgeGeometry =
    | { id: number; kind: 'curve'; from: Vec2; to: Vec2 }
    | {
          id: number;
          kind: 'elbow';
          from: Vec2;
          to: Vec2;
          fromNormal: Vec2;
          toNormal: Vec2;
          bend: number | null;
      };
