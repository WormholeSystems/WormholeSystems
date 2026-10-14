import { describe, expect, it } from 'vitest';

import { MapCamera } from './map-camera.svelte';

function camera() {
	const cam = new MapCamera(1);
	cam.viewportSize = { width: 800, height: 600 };
	return cam;
}

describe('zoomBy', () => {
	it('keeps the world point under the viewport center where it is', () => {
		const cam = camera();
		cam.pan = { x: -100, y: -50 };
		const before = cam.toWorld(400, 300);
		cam.zoomBy(1);
		expect(cam.zoom).toBeCloseTo(1.1);
		const after = cam.toWorld(400, 300);
		expect(after.x).toBeCloseTo(before.x);
		expect(after.y).toBeCloseTo(before.y);
	});

	it('holds the center across a zoom out and back in', () => {
		const cam = camera();
		cam.pan = { x: 37, y: -240 };
		const before = cam.toWorld(400, 300);
		cam.zoomBy(-3);
		cam.zoomBy(3);
		expect(cam.zoom).toBeCloseTo(1);
		const after = cam.toWorld(400, 300);
		expect(after.x).toBeCloseTo(before.x);
		expect(after.y).toBeCloseTo(before.y);
	});

	it('clamps at the top of the range', () => {
		const cam = camera();
		cam.zoomBy(100);
		expect(cam.zoom).toBe(2);
	});

	it('clamps at the bottom of the range', () => {
		const cam = camera();
		cam.zoomBy(-100);
		expect(cam.zoom).toBe(0.5);
	});

	it('does not move the view when already at the limit', () => {
		const cam = camera();
		cam.zoomBy(100);
		const pan = { ...cam.pan };
		cam.zoomBy(1);
		expect(cam.pan).toEqual(pan);
	});

	it('steps in tenths, so repeated steps land on round levels', () => {
		const cam = camera();
		cam.zoomBy(-1);
		cam.zoomBy(-1);
		expect(cam.zoom).toBeCloseTo(0.8);
	});
});

describe('toWorld', () => {
	it('inverts pan and zoom', () => {
		const cam = camera();
		cam.pan = { x: 50, y: 20 };
		cam.zoom = 2;
		expect(cam.toWorld(250, 120)).toEqual({ x: 100, y: 50 });
	});
});

describe('panBy', () => {
	it('moves by screen pixels regardless of zoom', () => {
		const cam = camera();
		cam.zoom = 2;
		cam.panBy(10, -5);
		expect(cam.pan).toEqual({ x: 10, y: -5 });
	});

	it('wakes the scrollbars', () => {
		const cam = camera();
		expect(cam.scrollbarsVisible).toBe(false);
		cam.panBy(1, 0);
		expect(cam.scrollbarsVisible).toBe(true);
	});
});

describe('viewportRect', () => {
	it('reports the observed size with no element attached', () => {
		const cam = camera();
		expect(cam.viewportRect()).toEqual({ left: 0, top: 0, width: 800, height: 600 });
	});
});

describe('screen origin', () => {
	function attached(left: number, top: number) {
		const cam = camera();
		let reads = 0;
		const rect = { left, top };
		cam.viewportEl = {
			getBoundingClientRect: () => {
				reads++;
				return rect;
			},
		} as unknown as HTMLElement;
		return { cam, rect, reads: () => reads };
	}

	it('measures the canvas once and reuses it across moves', () => {
		const { cam, reads } = attached(10, 20);
		expect(cam.toWorld(110, 220)).toEqual({ x: 100, y: 200 });
		cam.toWorld(0, 0);
		cam.viewportRect();
		expect(reads()).toBe(1);
	});

	it('measures again after being told the canvas may have moved', () => {
		const { cam, rect, reads } = attached(10, 20);
		cam.toWorld(0, 0);
		rect.left = 50;
		cam.remeasure();
		expect(cam.toWorld(150, 20)).toEqual({ x: 100, y: 0 });
		expect(reads()).toBe(2);
	});
});
