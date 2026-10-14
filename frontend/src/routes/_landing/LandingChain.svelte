<script lang="ts">
	// A real chain: the map's own SystemNode on the map's own grid, wired by the map's own
	// edge router. Nothing here is a drawing of the product, so nothing here can drift
	// away from it.
	import { treeEdges } from '$lib/map/edges';
	import { gridBackground } from '$lib/map/background';
	import SystemNode from '$lib/components/SystemNode.svelte';
	import { DEMO_CONNECTIONS, DEMO_PILOTS, DEMO_SYSTEMS } from './demo-chain';

	const CELL = 20;
	const NODE_H = 2 * CELL;
	// Whole cells, so the nodes stay on the grid lines they are padded away from.
	const PAD = 3 * CELL;
	const WIDTH = 700 + 2 * PAD;
	const HEIGHT = 380 + 2 * PAD;

	const positions = new Map(
		DEMO_SYSTEMS.map((s) => [s.id, { x: s.position_x + PAD, y: s.position_y + PAD }]),
	);
	const edges = [...treeEdges(DEMO_CONNECTIONS, positions, NODE_H).values()];

	const noop = () => {};

	function sigCounts(id: number) {
		// Turnur is the one with a scan behind it, so it is the one that gets the icon.
		return id === 1
			? { total: 4, uncategorized: 1, wormholes: 2 }
			: { total: 0, uncategorized: 0, wormholes: 0 };
	}

	function connectionCount(id: number) {
		return DEMO_CONNECTIONS.filter((c) => c.from_system === id || c.to_system === id).length;
	}
</script>

<!-- Scales as one piece rather than reflowing, the way a map does: the canvas keeps its
     own pixel geometry and the wrapper takes the scaled size so the layout still fits. -->
<div class="chain-fit" style:--w="{WIDTH}px" style:--h="{HEIGHT}px" aria-hidden="true">
	<div class="relative origin-top-left" style:width="{WIDTH}px" style:height="{HEIGHT}px">
		<div
			class="absolute -inset-96 opacity-70"
			style:background-image={gridBackground()}
			style:background-size="{CELL}px {CELL}px"
		></div>

		<svg class="absolute inset-0 overflow-visible" width={WIDTH} height={HEIGHT}>
			{#each edges as edge, i (edge.id)}
				<path
					class="landing-edge"
					d={edge.d}
					fill="none"
					stroke="var(--color-border)"
					stroke-width="1.5"
					style:--delay="{i * 90}ms"
				/>
			{/each}
		</svg>

		{#each DEMO_SYSTEMS as node, i (node.id)}
			<div class="landing-node" style:--delay="{i * 90}ms">
				<SystemNode
					{node}
					nodeH={NODE_H}
					pos={positions.get(node.id)!}
					selected={false}
					sigCounts={sigCounts(node.id)}
					connectionCount={connectionCount(node.id)}
					pilots={DEMO_PILOTS[node.id] ?? []}
					draggable={false}
					linkable={false}
					editable={false}
					onselect={noop}
					ondown={noop}
					onlink={noop}
					onmenu={noop}
					onsavealias={noop}
				/>
			</div>
		{/each}
	</div>
</div>

<style>
	.chain-fit {
		--s: 1;
		width: calc(var(--w) * var(--s));
		height: calc(var(--h) * var(--s));
	}

	.chain-fit > :global(div) {
		transform: scale(var(--s));
	}

	/* Beside the copy the canvas gets about 590px of a max-w-7xl row; stacked below that it
	   has the whole width, until the viewport itself is narrower than the canvas. */
	@media (min-width: 1280px) {
		.chain-fit {
			--s: 0.74;
		}
	}

	@media (max-width: 767px) {
		.chain-fit {
			--s: 0.56;
		}
	}

	@media (max-width: 479px) {
		.chain-fit {
			--s: 0.42;
		}
	}

	/* The chain assembles itself once, in chain order. Visible is the resting state, so a
	   suppressed animation leaves the map drawn rather than empty. */
	.landing-node {
		animation: node-in 400ms ease-out backwards;
		animation-delay: var(--delay);
	}

	@keyframes node-in {
		from {
			opacity: 0;
			transform: translateY(4px);
		}
		to {
			opacity: 1;
			transform: none;
		}
	}

	.landing-edge {
		animation: draw 700ms ease-out backwards;
		animation-delay: var(--delay);
	}

	@keyframes draw {
		from {
			stroke-dasharray: 600;
			stroke-dashoffset: 600;
		}
		to {
			stroke-dasharray: 600;
			stroke-dashoffset: 0;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.landing-node,
		.landing-edge {
			animation: none;
		}
	}
</style>
