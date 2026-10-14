// The pure math under the map's pointer gestures: where a drag puts every co-dragged
// node, and what a rubber band selects.

import { NODE_W, type Vec2 } from './geometry';

/**
 * A live drag. `members` are the co-dragged nodes with their start top-left; each moves by
 * the same delta the primary moved.
 */
export interface Drag {
	primary: number;
	x: number;
	y: number;
	offX: number;
	offY: number;
	members: { id: number; sx: number; sy: number }[];
}

/** Where every member sits now: each offset by the delta the primary has moved. */
export function draggedPositions(drag: Drag): Map<number, Vec2> {
	const start = drag.members.find((m) => m.id === drag.primary);
	const dx = drag.x - (start?.sx ?? drag.x);
	const dy = drag.y - (start?.sy ?? drag.y);
	return new Map(drag.members.map((m) => [m.id, { x: m.sx + dx, y: m.sy + dy }]));
}

export interface Band {
	x0: number;
	y0: number;
	x1: number;
	y1: number;
}

/** The nodes whose center the band covers. `nodes` carry their rendered top-left. */
export function bandSelection(
	band: Band,
	nodes: { id: number; x: number; y: number }[],
	nodeH: number,
): Set<number> {
	const loX = Math.min(band.x0, band.x1);
	const hiX = Math.max(band.x0, band.x1);
	const loY = Math.min(band.y0, band.y1);
	const hiY = Math.max(band.y0, band.y1);
	const hit = nodes.filter((n) => {
		const cx = n.x + NODE_W / 2;
		const cy = n.y + nodeH / 2;
		return cx >= loX && cx <= hiX && cy >= loY && cy <= hiY;
	});
	return new Set(hit.map((n) => n.id));
}

/** A placement as the drag math needs it: where it sits, and whether it may move. */
export interface DragCandidate {
	id: number;
	position_x: number;
	position_y: number;
	is_pinned: boolean;
}

/**
 * Which nodes one grab drags: the whole (non-pinned) selection when the grabbed node is
 * part of one, otherwise just the node itself at where it currently renders.
 */
export function dragMembers(
	grabbed: { id: number; at: Vec2 },
	selected: Set<number>,
	systems: DragCandidate[],
	pending: Record<number, Vec2>,
): { id: number; sx: number; sy: number }[] {
	if (!selected.has(grabbed.id) || selected.size < 2) {
		return [{ id: grabbed.id, sx: grabbed.at.x, sy: grabbed.at.y }];
	}
	const posOf = (id: number): Vec2 | null => {
		const p = pending[id];
		if (p) return p;
		const sys = systems.find((x) => x.id === id);
		return sys ? { x: sys.position_x, y: sys.position_y } : null;
	};
	return [...selected]
		.filter((id) => !systems.some((x) => x.id === id && x.is_pinned))
		.flatMap((id) => {
			const p = posOf(id);
			return p ? [{ id, sx: p.x, sy: p.y }] : [];
		});
}
