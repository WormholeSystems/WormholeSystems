import { describe, expect, it } from 'vitest';

import type { MapConnection } from '$lib/api/types/MapConnection';
import { LIFETIME_OPTIONS, MASS_OPTIONS } from './connection-status';
import { edgeDecorations, freeEdgeRouter, freeEdges, treeEdges } from './edges';
import { NODE_W } from './geometry';

const NODE_H = 40;

function connection(id: number, from: number, to: number): MapConnection {
	return { id, from_system: from, to_system: to } as MapConnection;
}

function wormhole(state: Partial<MapConnection>): MapConnection {
	return {
		kind: 'wormhole',
		mass_status: null,
		time_status: null,
		size: null,
		...state,
	} as MapConnection;
}

const massColor = (value: string) => MASS_OPTIONS.find((o) => o.value === value)!.color;
const timeColor = (value: string) => LIFETIME_OPTIONS.find((o) => o.value === value)!.color;

describe('edgeDecorations', () => {
	it('draws a fresh wormhole plain: solid, no badges', () => {
		expect(edgeDecorations(wormhole({}))).toEqual({
			dashed: false,
			pulsing: false,
			massColor: null,
			timeColor: null,
			sizeLabel: null,
			badgeCount: 0,
			badgeWidth: 8,
		});
	});

	it('treats stable statuses as no badge at all', () => {
		const deco = edgeDecorations(wormhole({ mass_status: 'stable', time_status: 'stable' }));
		expect(deco.dashed).toBe(false);
		expect(deco.badgeCount).toBe(0);
	});

	it('dashes every degraded state', () => {
		expect(edgeDecorations(wormhole({ mass_status: 'reduced' })).dashed).toBe(true);
		expect(edgeDecorations(wormhole({ mass_status: 'critical' })).dashed).toBe(true);
		expect(edgeDecorations(wormhole({ time_status: 'eol' })).dashed).toBe(true);
		expect(edgeDecorations(wormhole({ time_status: 'critical' })).dashed).toBe(true);
		expect(edgeDecorations(wormhole({ time_status: 'expired' })).dashed).toBe(true);
	});

	it('pulses an expired hole, and only that, in red', () => {
		const expired = edgeDecorations(wormhole({ time_status: 'expired' }));
		expect(expired.pulsing).toBe(true);
		expect(expired.timeColor).toBe(timeColor('critical'));
		expect(edgeDecorations(wormhole({ time_status: 'critical' })).pulsing).toBe(false);
	});

	it('never dashes a stargate, whatever its statuses say', () => {
		const gate = wormhole({ kind: 'stargate', mass_status: 'critical' });
		expect(edgeDecorations(gate).dashed).toBe(false);
	});

	it('takes the badge colors from the shared status options', () => {
		const deco = edgeDecorations(wormhole({ mass_status: 'reduced', time_status: 'eol' }));
		expect(deco.massColor).toBe(massColor('reduced'));
		expect(deco.timeColor).toBe(timeColor('eol'));
	});

	it('labels every known size, unknown being the only quiet one', () => {
		expect(edgeDecorations(wormhole({ size: 'small' })).sizeLabel).toBe('S');
		expect(edgeDecorations(wormhole({ size: 'xl' })).sizeLabel).toBe('XL');
		expect(edgeDecorations(wormhole({ size: 'large' })).sizeLabel).toBe('L');
		expect(edgeDecorations(wormhole({ size: null })).sizeLabel).toBeNull();
	});

	it('leaves a gate unlabelled, whatever size the row happens to carry', () => {
		expect(edgeDecorations(wormhole({ kind: 'stargate', size: 'large' })).sizeLabel).toBeNull();
	});

	it('sizes the pill by how many badges it holds', () => {
		const gate = edgeDecorations(wormhole({ kind: 'stargate' }));
		expect(gate.badgeCount).toBe(1);
		expect(gate.badgeWidth).toBe(26);
		const busy = edgeDecorations(
			wormhole({ size: 'small', mass_status: 'critical', time_status: 'critical' }),
		);
		expect(busy.badgeCount).toBe(3);
		expect(busy.badgeWidth).toBe(3 * 18 + 8);
	});
});

describe('freeEdges', () => {
	it('pulls each endpoint along its node toward the other one', () => {
		const positions = new Map([
			[1, { x: 0, y: 0 }],
			[2, { x: 600, y: 0 }],
		]);
		const g = freeEdges([connection(10, 1, 2)], positions, NODE_H).get(10)!;

		expect(g.from.y).toBe(NODE_H / 2);
		expect(g.to.y).toBe(NODE_H / 2);
		expect(g.from.x).toBeGreaterThan(0);
		expect(g.to.x).toBeLessThan(600 + NODE_W);
		expect(g.d.startsWith('M ')).toBe(true);
	});

	it('drops a connection whose endpoint is not placed', () => {
		const positions = new Map([[1, { x: 0, y: 0 }]]);
		expect(freeEdges([connection(10, 1, 2)], positions, NODE_H).size).toBe(0);
	});

	it('fans two holes between the same pair apart, whichever way round they were drawn', () => {
		const positions = new Map([
			[1, { x: 0, y: 0 }],
			[2, { x: 600, y: 0 }],
		]);
		const edges = freeEdges([connection(10, 1, 2), connection(11, 2, 1)], positions, NODE_H);
		const first = edges.get(10)!;
		const second = edges.get(11)!;

		expect(first.from.y).not.toBe(second.to.y);
		// Symmetrical about the rail the single edge would have used.
		expect(first.from.y + second.to.y).toBe(NODE_H);
	});
});

describe('treeEdges', () => {
	it('leaves the facing sides when the nodes are in different columns', () => {
		const positions = new Map([
			[1, { x: 60, y: 40 }],
			[2, { x: 380, y: 40 }],
		]);
		const g = treeEdges([connection(10, 1, 2)], positions, NODE_H).get(10)!;

		// Out of the right edge of the first, into the left edge of the second.
		expect(g.from).toEqual({ x: 60 + NODE_W, y: 60 });
		expect(g.to).toEqual({ x: 380, y: 60 });
	});

	it('spreads the endpoints of edges that share a node edge', () => {
		const positions = new Map([
			[1, { x: 60, y: 200 }],
			[2, { x: 380, y: 40 }],
			[3, { x: 380, y: 360 }],
		]);
		const routed = treeEdges([connection(10, 1, 2), connection(11, 1, 3)], positions, NODE_H);

		const up = routed.get(10)!;
		const down = routed.get(11)!;
		// Both leave the same side of node 1, so they cannot leave from the same point.
		expect(up.from.x).toBe(down.from.x);
		expect(up.from.y).not.toBe(down.from.y);
		// The one going up leaves above the one going down.
		expect(up.from.y).toBeLessThan(down.from.y);
	});

	it('staggers the vertical runs so a fan nests instead of stacking', () => {
		const positions = new Map([
			[1, { x: 60, y: 200 }],
			[2, { x: 380, y: 40 }],
			[3, { x: 380, y: 120 }],
			[4, { x: 380, y: 360 }],
		]);
		const routed = treeEdges(
			[connection(10, 1, 2), connection(11, 1, 3), connection(12, 1, 4)],
			positions,
			NODE_H,
		);

		// The bend is the x of the vertical run; three edges off one node, three lanes.
		const bends = [10, 11, 12].map((id) => {
			const path = routed.get(id)!.d;
			return path;
		});
		expect(new Set(bends).size).toBe(3);
		// Every path is an elbow: it turns, so it has quadratic corners.
		for (const d of bends) expect(d).toContain('Q');
	});

	it('detours into the lane when nodes sit between the two in a column', () => {
		// Four pinned roots in the first column, and a connection joining the outer two:
		// straight down would vanish behind the two in between.
		const positions = new Map([
			[1, { x: 60, y: 40 }],
			[2, { x: 60, y: 160 }],
			[3, { x: 60, y: 280 }],
			[4, { x: 60, y: 400 }],
		]);
		const g = treeEdges([connection(10, 1, 4)], positions, NODE_H).get(10)!;

		// Both ends leave the same side, so the run happens beside the column.
		expect(g.from.x).toBe(60 + NODE_W);
		expect(g.to.x).toBe(60 + NODE_W);
		expect(g.center.x).toBeGreaterThan(60 + NODE_W);
		// And the run clears the nodes it was cutting through.
		expect(g.center.y).toBeGreaterThan(60);
		expect(g.center.y).toBeLessThan(420);
	});

	it('keeps a straight run when the column between them is clear', () => {
		const positions = new Map([
			[1, { x: 60, y: 40 }],
			[2, { x: 60, y: 400 }],
		]);
		const g = treeEdges([connection(10, 1, 2)], positions, NODE_H).get(10)!;

		// Nothing in the way, so it takes the short way: out of the bottom, into the top.
		expect(g.from).toEqual({ x: 60 + NODE_W / 2, y: 80 });
		expect(g.to).toEqual({ x: 60 + NODE_W / 2, y: 400 });
	});

	it('never runs vertically through a column it is only passing', () => {
		// Columns two apart: the midpoint between them is the column in between, which is
		// exactly where a naive bend would put the vertical run.
		const positions = new Map([
			[1, { x: 60, y: 40 }],
			[2, { x: 380, y: 300 }],
			[3, { x: 700, y: 40 }],
		]);
		const g = treeEdges([connection(10, 1, 3)], positions, NODE_H).get(10)!;

		const run = g.center.x;
		expect(run).toBeGreaterThan(60 + NODE_W);
		expect(run < 380 || run > 380 + NODE_W).toBe(true);
	});

	it('goes out of the top or bottom when the nodes share a column', () => {
		const positions = new Map([
			[1, { x: 60, y: 40 }],
			[2, { x: 60, y: 400 }],
		]);
		const g = treeEdges([connection(10, 1, 2)], positions, NODE_H).get(10)!;

		expect(g.from).toEqual({ x: 60 + NODE_W / 2, y: 80 });
		expect(g.to).toEqual({ x: 60 + NODE_W / 2, y: 400 });
	});
});

describe('a hub with many children in the next column', () => {
	const NODE_H2 = 40;
	const targetX = NODE_W + 136;
	const ys = [20, 80, 147, 207, 267, 327, 387, 447, 507, 567, 627, 687, 747];

	/** Every vertical segment of every edge, which is what a lane actually is. */
	function verticalRuns(edges: ReturnType<typeof treeEdges>) {
		const runs: { id: number; x: number; y1: number; y2: number }[] = [];
		for (const e of edges.values()) {
			const pts = [...e.d.matchAll(/[MLQ] (-?[\d.]+) (-?[\d.]+)/g)].map((m) => ({
				x: Number(m[1]),
				y: Number(m[2]),
			}));
			for (let i = 1; i < pts.length; i++) {
				const a = pts[i - 1];
				const b = pts[i];
				if (Math.abs(a.x - b.x) < 0.5 && Math.abs(a.y - b.y) > 0.5) {
					runs.push({ id: e.id, x: a.x, y1: Math.min(a.y, b.y), y2: Math.max(a.y, b.y) });
				}
			}
		}
		return runs;
	}

	function hub() {
		const positions = new Map([[1, { x: 0, y: 390 }]]);
		ys.forEach((y, i) => positions.set(i + 2, { x: targetX, y }));
		return treeEdges(
			ys.map((_, i) => connection(10 + i, 1, i + 2)),
			positions,
			NODE_H2,
		);
	}

	// The bend used to step a fixed distance from the middle, which walked the outermost
	// runs onto the target column: they were then shunted past it and doubled back.
	it('keeps every run between the two columns', () => {
		for (const edge of hub().values()) {
			for (const [, x] of edge.d.matchAll(/[MLQ] (-?[\d.]+) -?[\d.]+/g)) {
				expect(Number(x)).toBeGreaterThanOrEqual(NODE_W - 0.5);
				expect(Number(x)).toBeLessThanOrEqual(targetX + 0.5);
			}
		}
	});

	it('never draws two runs on top of each other', () => {
		const runs = verticalRuns(hub());
		for (let i = 0; i < runs.length; i++) {
			for (let j = i + 1; j < runs.length; j++) {
				if (runs[i].id === runs[j].id) continue;
				if (Math.abs(runs[i].x - runs[j].x) > 0.5) continue;
				const lo = Math.max(runs[i].y1, runs[j].y1);
				const hi = Math.min(runs[i].y2, runs[j].y2);
				expect(hi - lo).toBeLessThanOrEqual(0.5);
			}
		}
	});

	// Thirteen holes, six of them above the node and six below. Every run above overlaps
	// every other run above near the node, so six lines is the fewest it can be drawn with.
	it('uses no more lines than the runs actually need', () => {
		expect(new Set(verticalRuns(hub()).map((r) => Math.round(r.x))).size).toBe(6);
	});
});

describe('runs share a lane where they can', () => {
	const NODE_H2 = 40;
	const COL = NODE_W + 136;

	// The corner is the Q control point. The L before it is pulled back by the corner
	// radius, which varies with the segment lengths, so reading that instead makes two runs
	// on one lane look like two lanes.
	function bendsOf(edges: ReturnType<typeof treeEdges>) {
		return [...edges.values()].map((e) =>
			Math.round(Number([...e.d.matchAll(/Q (-?[\d.]+) /g)][0][1])),
		);
	}

	// A run up and a run down never overlap, so they can sit on the same line and still be
	// told apart: each keeps its own stroke. Giving them separate lines made a two-hole
	// system look like a ladder.
	it('puts one child above and one below on the same line', () => {
		const positions = new Map([
			[1, { x: 0, y: 300 }],
			[2, { x: COL, y: 180 }],
			[3, { x: COL, y: 420 }],
		]);
		const bends = bendsOf(
			treeEdges([connection(10, 1, 2), connection(11, 1, 3)], positions, NODE_H2),
		);
		expect(new Set(bends).size).toBe(1);
	});

	// Two runs the same way do overlap, and would hide each other.
	it('keeps two children on the same side apart', () => {
		const positions = new Map([
			[1, { x: 0, y: 300 }],
			[2, { x: COL, y: 60 }],
			[3, { x: COL, y: 180 }],
		]);
		const bends = bendsOf(
			treeEdges([connection(10, 1, 2), connection(11, 1, 3)], positions, NODE_H2),
		);
		expect(new Set(bends).size).toBe(2);
	});

	// A tree layout centres a parent between its two children, so the gap either side is
	// small. Every such parent in a column should kink on the same line: a map of them
	// kinking a few pixels apart reads as noise.
	it('lines up the kink for every parent in a column', () => {
		const positions = new Map<number, { x: number; y: number }>();
		const conns: MapConnection[] = [];
		let id = 0;
		let next = 1;
		[202, 321, 441, 561, 681].forEach((py, i) => {
			const parent = next++;
			positions.set(parent, { x: 687, y: py });
			for (const cy of [161 + i * 120, 221 + i * 120]) {
				const child = next++;
				positions.set(child, { x: 1007, y: cy });
				conns.push(connection(id++, parent, child));
			}
		});
		expect(new Set(bendsOf(treeEdges(conns, positions, NODE_H2))).size).toBe(1);
	});
});

// Distilled from a live map: a leftward edge into the left column ended level with a
// rightward edge into the right column, and packing order alone put the leftward lane
// right of the rightward one, so their level tails overlapped between the two bends.
describe('level tails from opposite sides stay apart', () => {
	it('keeps the leftward lane left of the rightward lane', () => {
		const positions = new Map([
			[1, { x: 0, y: 980 }], // hub with two children, its ports spread
			[2, { x: 320, y: 420 }],
			[3, { x: 320, y: 1040 }],
			[4, { x: 320, y: 620 }], // sends an edge back left, level with the hub's lower child
			[5, { x: 0, y: 1040 }],
		]);
		const g = treeEdges(
			[connection(20, 1, 2), connection(21, 1, 3), connection(22, 4, 5)],
			positions,
			40,
		);
		const bend = (id: number) => Number([...g.get(id)!.d.matchAll(/Q (-?[\d.]+) /g)][0][1]);

		// Both end at the same y on opposite faces of the corridor, in different lanes.
		expect(g.get(21)!.to.y).toBe(g.get(22)!.to.y);
		// The tails [near, leftward.bend] and [rightward.bend, far] must not overlap.
		expect(bend(22)).toBeLessThan(bend(21));
	});
});

// A node with two holes spreads its ends apart so they can be told apart. A neighbour level
// with it, holding only this one hole, has nothing to spread, so the two ends used to sit a
// few pixels apart and the line kinked on its way across for no reason.
describe('a level run stays straight', () => {
	const NODE_H2 = 40;

	it('follows the busier end rather than kinking into the middle of the quiet one', () => {
		const positions = new Map([
			[1, { x: 0, y: 300 }],
			[2, { x: 400, y: 300 }],
			[3, { x: 400, y: 460 }],
		]);
		const g = treeEdges([connection(10, 1, 2), connection(11, 1, 3)], positions, NODE_H2);
		const level = g.get(10)!;
		expect(level.from.y).toBe(level.to.y);
		expect(level.kind).toBe('elbow');
	});

	// Only a run that would otherwise be straight gets this: a node above or below still
	// has a real bend to make, and pulling its end across would drag the line off its own
	// centre line for nothing.
	it('leaves a run alone when the nodes are not level', () => {
		const positions = new Map([
			[1, { x: 0, y: 300 }],
			[2, { x: 400, y: 340 }],
			[3, { x: 400, y: 460 }],
		]);
		const g = treeEdges([connection(10, 1, 2), connection(11, 1, 3)], positions, NODE_H2);
		expect(g.get(10)!.to.y).toBe(340 + NODE_H2 / 2);
	});
});

describe('freeEdgeRouter', () => {
	it('reroutes only the edges whose ends moved', () => {
		const route = freeEdgeRouter();
		const a = { x: 0, y: 0 };
		const b = { x: 300, y: 0 };
		const c = { x: 600, y: 0 };
		const connections = [connection(1, 1, 2), connection(2, 2, 3), connection(3, 1, 3)];
		const first = route(
			connections,
			new Map([
				[1, a],
				[2, b],
				[3, c],
			]),
			NODE_H,
		);
		const after = new Map([
			[1, a],
			[2, b],
			[3, { x: 600, y: 200 }],
		]);
		const second = route(connections, after, NODE_H);
		expect(second.get(1)).toBe(first.get(1));
		expect(second.get(2)).not.toBe(first.get(2));
		expect(second.get(3)).not.toBe(first.get(3));
		// The same answer the stateless router gives.
		expect([...second.values()]).toEqual([...freeEdges(connections, after, NODE_H).values()]);
	});

	it('hands back the same map when nothing moved', () => {
		const route = freeEdgeRouter();
		const positions = new Map([
			[1, { x: 0, y: 0 }],
			[2, { x: 300, y: 0 }],
		]);
		const first = route([connection(1, 1, 2)], positions, NODE_H);
		expect(route([connection(1, 1, 2)], positions, NODE_H)).toBe(first);
	});
});
