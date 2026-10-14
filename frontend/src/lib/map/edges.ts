// Edge geometry for the whole map at once: the tree router has to see every edge to fan
// out the ones sharing a node edge. All world units; the canvas transform does the scaling.

import type { MapConnection } from '$lib/api/types/MapConnection';
import { massBadgeColor, timeBadgeColor } from './connection-status';
import { NODE_W, railEndpoint, type Vec2 } from './geometry';
import { sizeLetter } from './mass';

export type { Vec2 };

/** Everything a connection's state hangs on its line: the dash and the badge pill. */
export interface EdgeDecorations {
	/** Degraded wormholes dash. */
	dashed: boolean;
	/** An expired hole pulses, line and badge, since it can go between two glances. */
	pulsing: boolean;
	massColor: string | null;
	timeColor: string | null;
	/** The known size, or null while nobody has said. A gate carries no size to show. */
	sizeLabel: string | null;
	badgeCount: number;
	/** The pill's width, from how many badges it holds. */
	badgeWidth: number;
}

export function edgeDecorations(c: MapConnection): EdgeDecorations {
	const dashed =
		c.kind === 'wormhole' &&
		(c.mass_status === 'reduced' ||
			c.mass_status === 'critical' ||
			(c.time_status !== null && c.time_status !== 'stable'));
	const pulsing = c.kind === 'wormhole' && c.time_status === 'expired';
	const massColor = massBadgeColor(c.mass_status);
	const timeColor = timeBadgeColor(c.time_status);
	const sizeLabel = c.kind === 'wormhole' && c.size !== null ? sizeLetter(c.size) : null;
	const badgeCount =
		(c.kind === 'stargate' ? 1 : 0) +
		(sizeLabel ? 1 : 0) +
		(massColor ? 1 : 0) +
		(timeColor ? 1 : 0);
	return {
		dashed,
		pulsing,
		massColor,
		timeColor,
		sizeLabel,
		badgeCount,
		badgeWidth: badgeCount * 18 + 8,
	};
}

export interface EdgeGeometry {
	id: number;
	/** A curve stops short of the node on its rail and needs an endpoint dot; an elbow does not. */
	kind: 'curve' | 'elbow';
	from: Vec2;
	to: Vec2;
	/** Where the badge cluster hangs. */
	center: Vec2;
	d: string;
}

interface Rect {
	minX: number;
	minY: number;
	maxX: number;
	maxY: number;
	centerX: number;
	centerY: number;
}

// World units.
const PARALLEL_SPACING = 14;
const BEND_SPACING = 16;
const CORNER_RADIUS = 10;
const LANE_MARGIN = 40;

function rectAt(position: Vec2, nodeH: number): Rect {
	return {
		minX: position.x,
		minY: position.y,
		maxX: position.x + NODE_W,
		maxY: position.y + nodeH,
		centerX: position.x + NODE_W / 2,
		centerY: position.y + nodeH / 2,
	};
}

const midpoint = (from: Vec2, to: Vec2): Vec2 => ({
	x: (from.x + to.x) / 2,
	y: (from.y + to.y) / 2,
});

export function curveBetween(from: Vec2, to: Vec2): string {
	const cp1x = from.x + (to.x - from.x) / 1.5;
	const cp2x = to.x - (to.x - from.x) / 1.5;
	return `M ${from.x} ${from.y} C ${cp1x} ${from.y}, ${cp2x} ${to.y}, ${to.x} ${to.y}`;
}

/**
 * How far each edge is shifted off its rail so that two holes between the same pair of
 * systems read as two lines. Without it they are drawn exactly on top of each other, and a
 * chain with a double connection looks like it has one hole whose state keeps changing.
 */
function parallelOffsets(connections: MapConnection[], nodeH: number): Map<number, number> {
	const groups = new Map<string, number[]>();
	for (const c of connections) {
		const [low, high] =
			c.from_system < c.to_system ? [c.from_system, c.to_system] : [c.to_system, c.from_system];
		const key = `${low}:${high}`;
		groups.set(key, [...(groups.get(key) ?? []), c.id]);
	}
	const offsets = new Map<number, number>();
	for (const ids of groups.values()) {
		if (ids.length < 2) continue;
		// Never wider than the node, however many holes end up between one pair.
		const spacing = Math.min(PARALLEL_SPACING, (nodeH * 0.7) / (ids.length - 1));
		ids.forEach((id, i) => offsets.set(id, (i - (ids.length - 1) / 2) * spacing));
	}
	return offsets;
}

/** Endpoints slide along a rail through each node's centre line, pulled toward the other. */
export function freeEdges(
	connections: MapConnection[],
	positions: ReadonlyMap<number, Vec2>,
	nodeH: number,
): Map<number, EdgeGeometry> {
	return freeEdgeRouter()(connections, positions, nodeH);
}

/**
 * [`freeEdges`] that remembers what it drew. A free edge depends on nothing but its two
 * ends and its rail, so one whose ends are the same position objects as last time keeps
 * its old geometry: dragging one node reroutes the few edges that touch it rather than the
 * whole map, and an unchanged edge hands its component the same object, which then skips
 * its update. When nothing changed at all the previous map comes back as it was.
 */
export function freeEdgeRouter() {
	let drawn = new Map<number, { a: Vec2; b: Vec2; rail: number; geometry: EdgeGeometry }>();
	let last: Map<number, EdgeGeometry> | null = null;

	return (
		connections: MapConnection[],
		positions: ReadonlyMap<number, Vec2>,
		nodeH: number,
	): Map<number, EdgeGeometry> => {
		const out = new Map<number, EdgeGeometry>();
		const next: typeof drawn = new Map();
		const offsets = parallelOffsets(connections, nodeH);
		let reused = 0;
		for (const c of connections) {
			const a = positions.get(c.from_system);
			const b = positions.get(c.to_system);
			if (!a || !b) continue;
			const rail = nodeH / 2 + (offsets.get(c.id) ?? 0);
			const before = drawn.get(c.id);
			if (before && before.a === a && before.b === b && before.rail === rail) {
				next.set(c.id, before);
				out.set(c.id, before.geometry);
				reused++;
				continue;
			}
			const from = railEndpoint(a.x, a.x + NODE_W, a.y + rail, b.x + NODE_W / 2);
			const to = railEndpoint(b.x, b.x + NODE_W, b.y + rail, a.x + NODE_W / 2);
			const geometry: EdgeGeometry = {
				id: c.id,
				kind: 'curve',
				from,
				to,
				center: midpoint(from, to),
				d: curveBetween(from, to),
			};
			next.set(c.id, { a, b, rail, geometry });
			out.set(c.id, geometry);
		}
		drawn = next;
		if (last && reused === out.size && out.size === last.size) return last;
		last = out;
		return out;
	};
}

interface Routed {
	id: number;
	from: Vec2;
	to: Vec2;
	fromNormal: Vec2;
	toNormal: Vec2;
	source: Rect;
	target: Rect;
	bend: number | null;
	/** Both ends leave the same side, to get around the nodes between them. */
	detour: boolean;
	/** Perpendicular distance and signed offset of the far end, for ordering the fan. */
	distance: number;
	signed: number;
}

/**
 * Left and right edges win whenever the boxes are separated horizontally, so the long run
 * drops through the lane between columns rather than through the stacked siblings.
 * `detour` sends both ends out the same side, for two nodes in one column with something
 * between them.
 */
function facingEnds(source: Rect, target: Rect, detour: boolean) {
	if (detour) {
		return {
			from: { x: source.maxX, y: source.centerY },
			to: { x: target.maxX, y: target.centerY },
			fromNormal: { x: 1, y: 0 },
			toNormal: { x: 1, y: 0 },
		};
	}
	return facingEndsDirect(source, target);
}

function facingEndsDirect(source: Rect, target: Rect) {
	const dx = target.centerX - source.centerX;
	const dy = target.centerY - source.centerY;
	const separatedX = target.minX > source.maxX || source.minX > target.maxX;
	const separatedY = target.minY > source.maxY || source.minY > target.maxY;

	if (separatedX || (!separatedY && Math.abs(dx) >= Math.abs(dy))) {
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

/** Whether a node sits between these two in their shared column, which a run would cross. */
function blockedInColumn(source: Rect, target: Rect, column: Rect[]): boolean {
	const top = Math.min(source.centerY, target.centerY);
	const bottom = Math.max(source.centerY, target.centerY);
	return column.some(
		(other) => other !== source && other !== target && other.minY < bottom && other.maxY > top,
	);
}

/**
 * A vertical run belongs in the lanes between columns, never inside one: crossing another
 * edge is readable, disappearing behind a node is not. Only columns the run actually
 * passes between count, and the shifted lane stays inside them, so an edge never runs past
 * the node it is heading for and doubles back.
 */
function intoLane(x: number, columns: number[], from: number, to: number): number {
	const near = Math.min(from, to);
	const far = Math.max(from, to);
	for (const left of columns) {
		if (left + NODE_W <= near || left >= far) continue;
		if (x > left - LANE_MARGIN / 2 && x < left + NODE_W + LANE_MARGIN) {
			return clamp(left + NODE_W + LANE_MARGIN, near, far);
		}
	}
	return x;
}

const clamp = (v: number, min: number, max: number) => Math.min(Math.max(v, min), max);

interface Port {
	endpoint: Vec2;
	normal: Vec2;
	box: Rect;
	sortKey: number;
}

/** Spread shared endpoints along the node edge, ordered by the far end so lines never cross. */
function spreadSharedEdge(ports: Port[]): void {
	if (ports.length < 2) return;
	const alongY = ports[0].normal.x !== 0;
	const box = ports[0].box;
	const extent = alongY ? box.maxY - box.minY : box.maxX - box.minX;
	const spacing = Math.min(PARALLEL_SPACING, (extent * 0.7) / (ports.length - 1));
	ports.sort((a, b) => a.sortKey - b.sortKey);
	ports.forEach((port, i) => {
		const offset = (i - (ports.length - 1) / 2) * spacing;
		if (alongY) port.endpoint.y += offset;
		else port.endpoint.x += offset;
	});
}

/**
 * Positions for the lanes of one corridor honouring `precedes` (lane → lanes that must
 * sit farther out), keeping the original order where the constraints leave a choice.
 * A cycle would mean tails that cannot all be kept apart; the tie is broken in favour
 * of the earlier lane and the rest still honoured.
 */
function laneOrder(precedes: ReadonlySet<number>[]): number[] {
	const indegree = precedes.map(() => 0);
	for (const targets of precedes) {
		for (const target of targets) {
			indegree[target]++;
		}
	}
	const position = precedes.map(() => 0);
	const placed = new Set<number>();
	for (let slot = 0; slot < precedes.length; slot++) {
		let pick = -1;
		for (let lane = 0; lane < precedes.length; lane++) {
			if (placed.has(lane)) continue;
			if (indegree[lane] === 0) {
				pick = lane;
				break;
			}
			if (pick === -1) pick = lane;
		}
		placed.add(pick);
		position[pick] = slot;
		for (const target of precedes[pick]) {
			if (!placed.has(target)) indegree[target]--;
		}
	}
	return position;
}

function elbowCorners(edge: Routed): [Vec2, Vec2] {
	if (edge.fromNormal.x !== 0) {
		const midX = edge.bend ?? (edge.from.x + edge.to.x) / 2;
		return [
			{ x: midX, y: edge.from.y },
			{ x: midX, y: edge.to.y },
		];
	}
	const midY = edge.bend ?? (edge.from.y + edge.to.y) / 2;
	return [
		{ x: edge.from.x, y: midY },
		{ x: edge.to.x, y: midY },
	];
}

function roundedPath(points: Vec2[], radius: number): string {
	const pts = points.filter(
		(p, i) => i === 0 || Math.hypot(p.x - points[i - 1].x, p.y - points[i - 1].y) > 0.01,
	);
	if (pts.length < 2) return '';
	let d = `M ${pts[0].x} ${pts[0].y}`;
	for (let i = 1; i < pts.length - 1; i++) {
		const prev = pts[i - 1];
		const curr = pts[i];
		const next = pts[i + 1];
		const lenIn = Math.hypot(curr.x - prev.x, curr.y - prev.y);
		const lenOut = Math.hypot(next.x - curr.x, next.y - curr.y);
		const r = Math.min(radius, lenIn / 2, lenOut / 2);
		const start = {
			x: curr.x + ((prev.x - curr.x) / lenIn) * r,
			y: curr.y + ((prev.y - curr.y) / lenIn) * r,
		};
		const end = {
			x: curr.x + ((next.x - curr.x) / lenOut) * r,
			y: curr.y + ((next.y - curr.y) / lenOut) * r,
		};
		d += ` L ${start.x} ${start.y} Q ${curr.x} ${curr.y} ${end.x} ${end.y}`;
	}
	const last = pts[pts.length - 1];
	return `${d} L ${last.x} ${last.y}`;
}

/**
 * A global pass rather than one edge at a time: both fanning shared endpoints and
 * staggering the runs that leave one node need the whole picture.
 */
export function treeEdges(
	connections: MapConnection[],
	positions: ReadonlyMap<number, Vec2>,
	nodeH: number,
): Map<number, EdgeGeometry> {
	// One rect per node, shared below so the column index can be compared by identity.
	const rects = new Map<number, Rect>();
	const byColumn = new Map<number, Rect[]>();
	for (const [id, position] of positions) {
		const rect = rectAt(position, nodeH);
		rects.set(id, rect);
		const column = byColumn.get(rect.minX);
		if (column) column.push(rect);
		else byColumn.set(rect.minX, [rect]);
	}
	const columns = [...byColumn.keys()].sort((a, b) => a - b);

	const routed: Routed[] = [];
	for (const c of connections) {
		const source = rects.get(c.from_system);
		const target = rects.get(c.to_system);
		if (!source || !target) continue;
		const detour =
			source.minX === target.minX &&
			blockedInColumn(source, target, byColumn.get(source.minX) ?? []);
		const ends = facingEnds(source, target, detour);
		routed.push({
			id: c.id,
			from: { ...ends.from },
			to: { ...ends.to },
			fromNormal: ends.fromNormal,
			toNormal: ends.toNormal,
			source,
			target,
			bend: null,
			detour,
			distance: 0,
			signed: 0,
		});
	}

	const shared = new Map<string, Port[]>();
	const register = (endpoint: Vec2, normal: Vec2, box: Rect, other: Rect) => {
		const key = `${box.centerX},${box.centerY}|${normal.x},${normal.y}`;
		const port: Port = {
			endpoint,
			normal,
			box,
			sortKey: normal.x !== 0 ? other.centerY : other.centerX,
		};
		const ports = shared.get(key);
		if (ports) ports.push(port);
		else shared.set(key, [port]);
	};
	for (const edge of routed) {
		register(edge.from, edge.fromNormal, edge.source, edge.target);
		register(edge.to, edge.toNormal, edge.target, edge.source);
	}
	const spreadCount = new Map<Vec2, number>();
	for (const ports of shared.values()) {
		spreadSharedEdge(ports);
		for (const port of ports) spreadCount.set(port.endpoint, ports.length);
	}

	// Two nodes sitting level would be joined by a straight line, except that spreading the
	// ports on the busier one lifts its end off the other's centre line and leaves a jog of
	// a few pixels. Where the quiet end is this edge's alone, it follows the busy one
	// instead: entering off-centre reads better than a kink that means nothing.
	for (const edge of routed) {
		if (edge.detour) continue;
		const horizontal = edge.fromNormal.x !== 0;
		const level = horizontal
			? Math.abs(edge.source.centerY - edge.target.centerY) < 0.5
			: Math.abs(edge.source.centerX - edge.target.centerX) < 0.5;
		if (!level) continue;
		const fromShared = spreadCount.get(edge.from) ?? 1;
		const toShared = spreadCount.get(edge.to) ?? 1;
		if (fromShared === toShared) continue;
		const [follow, lead] = fromShared < toShared ? [edge.from, edge.to] : [edge.to, edge.from];
		if (horizontal) follow.y = lead.y;
		else follow.x = lead.x;
	}

	// Grouped by the corridor the run crosses, not by the node it leaves: two runs at the
	// same offset in the same corridor are the same line, whichever nodes they belong to,
	// so they all have to be packed together or they lie on top of each other.
	const fans = new Map<string, Routed[]>();
	for (const edge of routed) {
		if (edge.detour) continue;
		const horizontal = edge.fromNormal.x !== 0;
		const sourceFirst = horizontal
			? edge.source.centerX <= edge.target.centerX
			: edge.source.centerY <= edge.target.centerY;
		const primary = sourceFirst ? edge.source : edge.target;
		const other = sourceFirst ? edge.target : edge.source;
		const along = horizontal ? other.centerY - primary.centerY : other.centerX - primary.centerX;
		edge.distance = Math.abs(along);
		edge.signed = along;
		const across = horizontal ? [edge.from.x, edge.to.x] : [edge.from.y, edge.to.y];
		const key = `${horizontal ? 'h' : 'v'}|${Math.min(...across)},${Math.max(...across)}`;
		const group = fans.get(key);
		if (group) group.push(edge);
		else fans.set(key, [edge]);
	}
	// Runs leaving one node share the corridor to the next column. Two of them only need
	// separate lines where they would overlap and hide each other: a run up and a run down
	// can sit on the same line and still be told apart, because each keeps its own colour.
	// So pack them onto as few lines as possible, then space those across the corridor.
	for (const group of fans.values()) {
		if (group.length < 2) continue;
		const horizontal = group[0].fromNormal.x !== 0;
		// How far the run reaches along the node edge it leaves from.
		const reach = (e: Routed): [number, number] =>
			horizontal
				? [Math.min(e.from.y, e.to.y), Math.max(e.from.y, e.to.y)]
				: [Math.min(e.from.x, e.to.x), Math.max(e.from.x, e.to.x)];

		const lanes: Routed[][] = [];
		const laneOf = new Map<number, number>();
		const ordered = [...group].sort((a, b) => b.distance - a.distance || a.signed - b.signed);
		for (const edge of ordered) {
			const span = reach(edge);
			const fits = (lane: number) =>
				lanes[lane].every((other) => {
					const [s, e] = reach(other);
					return span[0] >= e || span[1] <= s;
				});
			// Overlap is the only thing a lane cannot have: two runs on one line hide each
			// other, where two runs that cross stay readable. Taken longest first, first fit,
			// that lands on the fewest lanes the corridor can be drawn with.
			let lane = lanes.findIndex((_, i) => fits(i));
			if (lane === -1) lane = lanes.push([]) - 1;
			lanes[lane].push(edge);
			laneOf.set(edge.id, lane);
		}

		// One corridor, so one set of lines: measured from its near edge, not from each
		// run's own direction, or a run drawn leftward would count its lanes backwards.
		const [near, far] = (() => {
			const ends = group.flatMap((e) => (horizontal ? [e.from.x, e.to.x] : [e.from.y, e.to.y]));
			return [Math.min(...ends), Math.max(...ends)];
		})();

		// Separate lanes only keep the runs themselves apart. Each run also has two tails
		// tying its lane to a corridor face, and the tails of a leftward and a rightward
		// edge that end level lie on one line: they stay apart only when the lane of the
		// edge entering the near face sits nearer than the lane of the edge entering the
		// far face. Renumber the lanes to honour those orderings, so a run never doubles
		// back over the tail of a level neighbour approaching from the other side.
		const tailsOf = (edge: Routed): { at: number; fromNear: boolean }[] => {
			const middle = (near + far) / 2;
			return horizontal
				? [
						{ at: edge.from.y, fromNear: edge.from.x < middle },
						{ at: edge.to.y, fromNear: edge.to.x < middle },
					]
				: [
						{ at: edge.from.x, fromNear: edge.from.y < middle },
						{ at: edge.to.x, fromNear: edge.to.y < middle },
					];
		};
		const precedes = lanes.map(() => new Set<number>());
		for (let i = 0; i < group.length; i++) {
			for (let j = i + 1; j < group.length; j++) {
				const laneA = laneOf.get(group[i].id)!;
				const laneB = laneOf.get(group[j].id)!;
				if (laneA === laneB) continue;
				for (const tailA of tailsOf(group[i])) {
					for (const tailB of tailsOf(group[j])) {
						if (Math.abs(tailA.at - tailB.at) >= 0.5 || tailA.fromNear === tailB.fromNear) continue;
						const [nearLane, farLane] = tailA.fromNear ? [laneA, laneB] : [laneB, laneA];
						precedes[nearLane].add(farLane);
					}
				}
			}
		}
		const position = laneOrder(precedes);

		for (const edge of group) {
			edge.bend = near + ((far - near) * (position[laneOf.get(edge.id)!] + 1)) / (lanes.length + 1);
		}
	}

	// Detours stack outwards so several between the same roots stay apart.
	const detours = new Map<number, Routed[]>();
	for (const edge of routed.filter((e) => e.detour)) {
		const group = detours.get(edge.source.minX);
		if (group) group.push(edge);
		else detours.set(edge.source.minX, [edge]);
	}
	for (const [columnX, group] of detours) {
		group.sort((a, b) => a.from.y - b.from.y);
		group.forEach((edge, i) => {
			edge.bend = columnX + NODE_W + LANE_MARGIN + i * BEND_SPACING;
		});
	}

	// The midpoint between two columns two apart lands exactly on the column between them.
	for (const edge of routed) {
		if (edge.detour || edge.fromNormal.x === 0) continue;
		edge.bend = intoLane(
			edge.bend ?? (edge.from.x + edge.to.x) / 2,
			columns,
			edge.from.x,
			edge.to.x,
		);
	}

	const out = new Map<number, EdgeGeometry>();
	for (const edge of routed) {
		const corners = elbowCorners(edge);
		out.set(edge.id, {
			id: edge.id,
			kind: 'elbow',
			from: edge.from,
			to: edge.to,
			center: midpoint(corners[0], corners[1]),
			d: roundedPath([edge.from, corners[0], corners[1], edge.to], CORNER_RADIUS),
		});
	}
	return out;
}
