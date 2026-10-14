// Turning pointers and arrow keys into grid placements. The algebra lives in
// `$lib/layout/grid`; this is the translation layer between events and it, pure so the
// hysteresis and snapping rules are testable without a pointer.

import { moveItem, resizeItem, type GridItem } from '$lib/layout/grid';
import { panelMeta, type PanelId } from './registry';

/** How far a pointer must travel before a press becomes a drag (matches the canvas). */
export const PANEL_HYSTERESIS = 4;

/**
 * `dx`/`dy` are the raw pixel offset, so the held tile tracks the cursor instead of
 * jumping a cell at a time. `live` is the snapped layout it would land in, which is what
 * the other tiles reflow to and what the placeholder shows.
 */
export interface PanelGesture {
	id: PanelId;
	kind: 'move' | 'resize';
	startX: number;
	startY: number;
	origin: GridItem;
	dx: number;
	dy: number;
	live: GridItem[] | null;
}

export interface GridMetrics {
	cols: number;
	rowHeight: number;
	colWidth: number;
	gridWidth: number;
}

export function beginGesture(
	id: PanelId,
	kind: 'move' | 'resize',
	at: { x: number; y: number },
	origin: GridItem,
): PanelGesture {
	return { id, kind, startX: at.x, startY: at.y, origin, dx: 0, dy: 0, live: null };
}

/**
 * The gesture after the pointer moved: unchanged under the hysteresis, otherwise carrying
 * the raw offset and the snapped layout it amounts to.
 */
export function moveGesture(
	gesture: PanelGesture,
	at: { x: number; y: number },
	items: GridItem[],
	metrics: GridMetrics,
): PanelGesture {
	const dx = at.x - gesture.startX;
	const dy = at.y - gesture.startY;
	if (!gesture.live && Math.hypot(dx, dy) < PANEL_HYSTERESIS) return gesture;

	const meta = panelMeta(gesture.id);
	const cols = Math.round(dx / metrics.colWidth);
	const rows = Math.round(dy / metrics.rowHeight);
	const live =
		gesture.kind === 'move'
			? moveItem(items, gesture.id, gesture.origin.x + cols, gesture.origin.y + rows, metrics.cols)
			: resizeItem(
					items,
					gesture.id,
					gesture.origin.w + cols,
					gesture.origin.h + rows,
					metrics.cols,
					meta,
				);
	return { ...gesture, dx, dy, live };
}

function clamp(v: number, lo: number, hi: number) {
	return Math.max(lo, Math.min(hi, v));
}

/** The dragged tile's free pixel box, following the pointer; null until the drag commits. */
export function floatingBox(
	gesture: PanelGesture,
	metrics: GridMetrics,
): { left: number; top: number; width: number; height: number } | null {
	if (!gesture.live) return null;
	const meta = panelMeta(gesture.id);
	const left = gesture.origin.x * metrics.colWidth;
	const top = gesture.origin.y * metrics.rowHeight;
	if (gesture.kind === 'move') {
		const width = gesture.origin.w * metrics.colWidth;
		// Held inside the grid: hanging off the right would widen the document and flash a
		// horizontal scrollbar mid-drag.
		return {
			left: clamp(left + gesture.dx, 0, Math.max(0, metrics.gridWidth - width)),
			top: Math.max(0, top + gesture.dy),
			width,
			height: gesture.origin.h * metrics.rowHeight,
		};
	}
	return {
		left,
		top,
		width: clamp(
			gesture.origin.w * metrics.colWidth + gesture.dx,
			meta.minW * metrics.colWidth,
			(metrics.cols - gesture.origin.x) * metrics.colWidth,
		),
		height: Math.max(
			gesture.origin.h * metrics.rowHeight + gesture.dy,
			meta.minH * metrics.rowHeight,
		),
	};
}

const KEY_DELTAS: Record<string, [number, number]> = {
	ArrowLeft: [-1, 0],
	ArrowRight: [1, 0],
	ArrowUp: [0, -1],
	ArrowDown: [0, 1],
};

/** Arrow keys move a focused tile, shift+arrows resize it; null for any other key. */
export function keyboardLayout(
	items: GridItem[],
	id: PanelId,
	key: string,
	resize: boolean,
	cols: number,
): GridItem[] | null {
	const delta = KEY_DELTAS[key];
	if (!delta) return null;
	const current = items.find((i) => i.i === id);
	if (!current) return null;
	const [dx, dy] = delta;
	return resize
		? resizeItem(items, id, current.w + dx, current.h + dy, cols, panelMeta(id))
		: moveItem(items, id, current.x + dx, current.y + dy, cols);
}
