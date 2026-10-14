// The canvas's pointer state machine. Gestures commit only after a few pixels of travel;
// until then a release counts as a tap. What a committed gesture does lives on the map
// state; this decides which gesture a pointer sequence was.

import type { GridConfig } from '$lib/api/types/GridConfig';
import type { MapSystemView } from '$lib/api/types/MapSystemView';
import { dragMembers, bandSelection, draggedPositions, type Drag } from '$lib/map/gestures';
import { nodeAt, type Vec2 } from '$lib/map/geometry';
import type { MapCamera } from './map-camera.svelte';

/** How far a press may wander (screen px) and still count as a tap. */
const HYSTERESIS = 4;

/**
 * What the gestures read and write on the map, plus the commands a finished gesture
 * issues. Narrow on purpose, like [`LayoutHost`]: a test hands in a plain object and a
 * real camera, and the commands carry no eagerly-built request promises.
 */
export interface GestureHost {
	camera: MapCamera;
	systems: {
		readonly all: MapSystemView[];
		move(moves: { map_solar_system_id: number; x: number; y: number }[]): void;
	};
	readonly positions: Map<number, Vec2>;
	readonly nodeH: number;
	readonly grid: GridConfig;
	readonly canWrite: boolean;
	readonly layoutLocked: boolean;
	selected: Set<number>;
	drag: Drag | null;
	pending: Record<number, Vec2>;
	linking: { from: number; x: number; y: number } | null;
	band: { x0: number; y0: number; x1: number; y1: number } | null;
	panDrag: { cx: number; cy: number; px: number; py: number } | null;
	connections: { add(from: number, to: number): void };
	snap(v: number): number;
	clampNodeX(x: number): number;
	clampNodeY(y: number): number;
	closeMenu(): void;
}

export class MapGestures {
	private map: GestureHost;
	private pendingDrag: { cx: number; cy: number; drag: Drag } | null = null;
	private pendingBand: { cx: number; cy: number } | null = null;

	constructor(map: GestureHost) {
		this.map = map;
	}

	private updateBandSelection() {
		const map = this.map;
		const b = map.band;
		if (!b) return;
		// Rendered positions, not stored ones: an automatic layout draws nodes elsewhere.
		const nodes = map.systems.all.map((s) => ({
			id: s.id,
			...(map.positions.get(s.id) ?? { x: s.position_x, y: s.position_y }),
		}));
		map.selected = bandSelection(b, nodes, map.nodeH);
	}

	/**
	 * Wheel over the canvas: plain scrolls the page, ctrl/meta is swallowed so a pinch does
	 * not zoom the whole app, shift pans the map.
	 */
	onWheel(ev: WheelEvent) {
		if (ev.ctrlKey || ev.metaKey) {
			ev.preventDefault();
			return;
		}
		if (!ev.shiftKey) return;
		ev.preventDefault();
		this.map.camera.panBy(-ev.deltaX, -ev.deltaY);
	}

	/** Listen for the wheel on `el`; returns the cleanup. Non-passive, to be allowed to preventDefault. */
	attachWheel(el: HTMLElement): () => void {
		const onWheel = (ev: WheelEvent) => this.onWheel(ev);
		el.addEventListener('wheel', onWheel, { passive: false });
		return () => el.removeEventListener('wheel', onWheel);
	}

	onPointerMove(ev: PointerEvent) {
		const map = this.map;
		const w = map.camera.toWorld(ev.clientX, ev.clientY);
		if (this.pendingDrag) {
			if (
				Math.hypot(ev.clientX - this.pendingDrag.cx, ev.clientY - this.pendingDrag.cy) >= HYSTERESIS
			) {
				map.drag = this.pendingDrag.drag;
				this.pendingDrag = null;
			} else {
				return;
			}
		}
		if (this.pendingBand) {
			if (
				Math.hypot(ev.clientX - this.pendingBand.cx, ev.clientY - this.pendingBand.cy) >= HYSTERESIS
			) {
				map.selected = new Set();
				const start = map.camera.toWorld(this.pendingBand.cx, this.pendingBand.cy);
				map.band = { x0: start.x, y0: start.y, x1: w.x, y1: w.y };
				this.pendingBand = null;
			} else {
				return;
			}
		}
		if (map.drag) {
			const d = map.drag;
			const nx = map.clampNodeX(map.snap(w.x - d.offX));
			const ny = map.clampNodeY(map.snap(w.y - d.offY));
			// Most moves stay inside one grid cell. A fresh object would still count as a
			// change and redraw every node and edge for nothing.
			if (nx === d.x && ny === d.y) return;
			map.drag = { ...d, x: nx, y: ny };
		} else if (map.linking) {
			map.linking = { ...map.linking, x: w.x, y: w.y };
		} else if (map.band) {
			map.band = { ...map.band, x1: w.x, y1: w.y };
			// The selection follows the band live.
			this.updateBandSelection();
		} else if (map.panDrag) {
			const p = map.panDrag;
			map.camera.pan = { x: p.px + ev.clientX - p.cx, y: p.py + ev.clientY - p.cy };
			map.camera.wakeScrollbars();
		}
	}

	onPointerUp(ev: PointerEvent) {
		const map = this.map;
		if (map.drag) {
			const d = map.drag;
			map.drag = null;
			const moves = [...draggedPositions(d)].map(([id, p]) => ({
				map_solar_system_id: id,
				x: p.x,
				y: p.y,
			}));
			// Seed the optimistic override before the refetch so nodes stay put.
			const pending = { ...map.pending };
			for (const m of moves) pending[m.map_solar_system_id] = { x: m.x, y: m.y };
			map.pending = pending;
			map.systems.move(moves);
		}
		if (map.linking) {
			const l = map.linking;
			map.linking = null;
			const w = map.camera.toWorld(ev.clientX, ev.clientY);
			const target = nodeAt(map.systems.all, w.x, w.y, map.grid, map.positions);
			// Dropping onto a ghost is the same claim from the other end, so it is no more
			// allowed than starting from one.
			const ghost = map.systems.all.some((s) => s.id === target && s.kind === 'ghost');
			if (target !== null && target !== l.from && !ghost) {
				map.connections.add(l.from, target);
			}
		}
		// A tap (no band committed) clears the selection.
		if (map.band) {
			map.band = null;
		} else if (this.pendingBand) {
			map.selected = new Set();
		}
		this.pendingBand = null;
		this.pendingDrag = null;
		map.panDrag = null;
	}

	// The pointer is captured only once an interaction starts: capturing on a right-button
	// press would retarget the upcoming contextmenu event away from the node under it.
	onBackgroundDown(ev: PointerEvent) {
		const map = this.map;
		// Every gesture measures the canvas once at its start; the moves reuse that.
		map.camera.remeasure();
		map.closeMenu();
		if (ev.button === 1) {
			ev.preventDefault();
			map.camera.viewportEl?.setPointerCapture(ev.pointerId);
			map.panDrag = { cx: ev.clientX, cy: ev.clientY, px: map.camera.pan.x, py: map.camera.pan.y };
		} else if (ev.button === 0) {
			map.camera.viewportEl?.setPointerCapture(ev.pointerId);
			// An automatic layout has nothing to drag, so a plain drag pans instead. A selection
			// modifier still belongs to the rubber band.
			const selecting = ev.shiftKey || ev.ctrlKey || ev.metaKey;
			if (map.layoutLocked && !selecting) {
				map.panDrag = {
					cx: ev.clientX,
					cy: ev.clientY,
					px: map.camera.pan.x,
					py: map.camera.pan.y,
				};
			} else {
				this.pendingBand = { cx: ev.clientX, cy: ev.clientY };
			}
		}
	}

	/** Co-drags the whole (non-pinned) selection when the grabbed node is part of one. */
	onNodeDown(ev: PointerEvent, s: MapSystemView) {
		const map = this.map;
		map.camera.remeasure();
		if (ev.button !== 0 || map.layoutLocked || !map.canWrite) return;
		ev.stopPropagation();
		map.closeMenu();

		const cur = map.positions.get(s.id) ?? { x: s.position_x, y: s.position_y };
		const members = dragMembers({ id: s.id, at: cur }, map.selected, map.systems.all, map.pending);

		if (!map.selected.has(s.id)) map.selected = new Set();
		if (s.is_pinned) return;
		map.camera.viewportEl?.setPointerCapture(ev.pointerId);
		// Record the grab offset so the node does not jump under the cursor.
		const w = map.camera.toWorld(ev.clientX, ev.clientY);
		this.pendingDrag = {
			cx: ev.clientX,
			cy: ev.clientY,
			drag: {
				primary: s.id,
				x: cur.x,
				y: cur.y,
				offX: w.x - cur.x,
				offY: w.y - cur.y,
				members,
			},
		};
	}

	onLinkDown(ev: PointerEvent, id: number) {
		const map = this.map;
		map.camera.remeasure();
		ev.stopPropagation();
		if (!map.canWrite) return;
		map.camera.viewportEl?.setPointerCapture(ev.pointerId);
		const w = map.camera.toWorld(ev.clientX, ev.clientY);
		map.linking = { from: id, x: w.x, y: w.y };
	}
}
