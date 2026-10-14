// The viewport: where the world sits on screen, how big it is drawn, and the scrollbars
// that show while it moves. Knows nothing about what is on the map.

import * as v from 'valibot';

import { readStored, writeStored } from '$lib/storage';
import { clamp, type Vec2 } from '$lib/map/geometry';

/** Half size is where node text stops being readable, double where a chain stops fitting. */
const ZOOM_MIN = 0.5;
const ZOOM_MAX = 2;
const ZOOM_STEP = 0.1;

/** How long the scrollbars stay up after the last thing that moved the view. */
const SCROLLBAR_LINGER_MS = 1500;

/** Where the canvas sits on screen, and how big it is. */
export interface ViewportRect {
	left: number;
	top: number;
	width: number;
	height: number;
}

export class MapCamera {
	private mapId: number;
	viewportEl: HTMLElement | null = null;

	pan = $state({ x: 0, y: 0 });
	zoom = $state(1);
	/** Shown while the scrollbars are awake; they fade out once nothing has moved. */
	scrollbarsVisible = $state(false);
	/** The canvas's rendered size, kept current by a ResizeObserver on the viewport. */
	viewportSize = $state({ width: 1200, height: 1400 });

	private hideScrollbars: ReturnType<typeof setTimeout> | null = null;

	constructor(mapId: number) {
		this.mapId = mapId;
	}

	/**
	 * Start measuring the canvas element; returns the cleanup. Its size is watched, since
	 * `getBoundingClientRect` is not reactive and the scrollbars derive from it, and so is
	 * anything that can move it on screen without resizing it: a page scroll (captured, so
	 * a scrolling panel counts) or a window resize.
	 */
	attach(el: HTMLElement): () => void {
		this.viewportEl = el;
		const observer = new ResizeObserver(([entry]) => {
			const box = entry.contentRect;
			this.viewportSize = { width: box.width, height: box.height };
			this.remeasure();
		});
		observer.observe(el);
		const moved = () => this.remeasure();
		window.addEventListener('scroll', moved, { capture: true, passive: true });
		window.addEventListener('resize', moved, { passive: true });
		return () => {
			observer.disconnect();
			window.removeEventListener('scroll', moved, { capture: true });
			window.removeEventListener('resize', moved);
			if (this.viewportEl === el) this.viewportEl = null;
		};
	}

	// Where the canvas sits on screen, measured once and kept until something may have moved
	// it. Reading it fresh on every pointer move would force a layout each time, right after
	// the previous move's DOM writes.
	private origin: { left: number; top: number } | null = null;

	/**
	 * Drop the measured screen position, so the next read measures again. Called where the
	 * canvas can have moved: a page scroll, a resize, and the start of every gesture.
	 */
	remeasure() {
		this.origin = null;
	}

	private screenOrigin(): { left: number; top: number } {
		if (this.origin) return this.origin;
		const el = this.viewportEl;
		if (!el) return { left: 0, top: 0 };
		const r = el.getBoundingClientRect();
		this.origin = { left: r.left, top: r.top };
		return this.origin;
	}

	viewportRect(): ViewportRect {
		const { left, top } = this.screenOrigin();
		return {
			left,
			top,
			// Size comes from the observer instead of the rect, because a rect read is not
			// reactive: anything derived from it (the scrollbar thumbs) would keep the
			// value it had when the canvas was first measured.
			width: this.viewportSize.width,
			height: this.viewportSize.height,
		};
	}

	/** Screen (client) point → world coords, accounting for pan + zoom. */
	toWorld(clientX: number, clientY: number): Vec2 {
		const r = this.screenOrigin();
		return {
			x: (clientX - r.left - this.pan.x) / this.zoom,
			y: (clientY - r.top - this.pan.y) / this.zoom,
		};
	}

	/** Shift the view by a screen-pixel delta (wheel, scrollbar, drag). */
	panBy(dx: number, dy: number) {
		this.pan = { x: this.pan.x + dx, y: this.pan.y + dy };
		this.wakeScrollbars();
	}

	/** Shown while the view is moving or the cursor is over the canvas, then faded out. */
	wakeScrollbars() {
		this.scrollbarsVisible = true;
		if (this.hideScrollbars) clearTimeout(this.hideScrollbars);
		this.hideScrollbars = setTimeout(() => {
			this.scrollbarsVisible = false;
			this.hideScrollbars = null;
		}, SCROLLBAR_LINGER_MS);
	}

	/** Zoom by whole steps, keeping the middle of the viewport where it is. */
	zoomBy(steps: number) {
		const next = Math.round((this.zoom + steps * ZOOM_STEP) * 10) / 10;
		const nz = clamp(next, ZOOM_MIN, ZOOM_MAX);
		if (nz === this.zoom) return;
		const z = this.zoom;
		const r = this.viewportRect();
		const cx = r.width / 2;
		const cy = r.height / 2;
		const wx = (cx - this.pan.x) / z;
		const wy = (cy - this.pan.y) / z;
		this.pan = { x: cx - wx * nz, y: cy - wy * nz };
		this.zoom = nz;
		this.rememberZoom();
		this.wakeScrollbars();
	}

	/** Per map and per browser: how far out you want to be depends on the screen. */
	restoreZoom() {
		// Legacy values were written as `String(zoom)`, which is valid JSON for a number.
		const saved = readStored(`map-zoom-${this.mapId}`, v.number(), NaN);
		if (saved >= ZOOM_MIN && saved <= ZOOM_MAX) this.zoom = saved;
	}

	private rememberZoom() {
		writeStored(`map-zoom-${this.mapId}`, this.zoom);
	}
}
