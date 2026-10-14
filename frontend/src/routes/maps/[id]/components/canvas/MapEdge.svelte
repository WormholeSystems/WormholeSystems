<script lang="ts">
	// One connection: its line, its state badges, and the wide invisible hit path.
	import ClockIcon from '@lucide/svelte/icons/clock';
	import OrbitIcon from '@lucide/svelte/icons/orbit';
	import WeightIcon from '@lucide/svelte/icons/weight';

	import type { MapConnection } from '$lib/api/types/MapConnection';
	import { edgeDecorations, type EdgeGeometry } from '$lib/map/edges';
	import { edgeColor } from '$lib/map/helpers';
	import { formatRemaining, lifetimeDeadline } from '$lib/map/lifetime';
	import type { MapState } from '../../state/map-state.svelte';

	let {
		map,
		connection,
		geometry,
		now,
	}: { map: MapState; connection: MapConnection; geometry: EdgeGeometry; now: number } = $props();

	const c = $derived(connection);
	const elbow = $derived(geometry.kind === 'elbow');
	const onRoute = $derived(map.route.connectionIds.has(c.id));
	const stroke = $derived(edgeColor(c.kind, c.mass_status, c.time_status, onRoute));
	const deco = $derived(edgeDecorations(c));
	// Only a mark's countdown earns ink on the line; the estimate stays in the popover. An
	// expired hole has nothing left to count, and its pulse already says so.
	const deadline = $derived(lifetimeDeadline(c));
	const timeLeft = $derived(
		deadline && !deadline.estimated && !deco.pulsing ? formatRemaining(deadline.at, now) : null,
	);
	const badgeWidth = $derived(deco.badgeWidth + (timeLeft ? timeLeft.length * 6 + 4 : 0));
</script>

<g class="group/edge">
	<path
		d={geometry.d}
		fill="none"
		{stroke}
		stroke-width={elbow ? 1.5 : 4}
		stroke-linecap="round"
		stroke-linejoin="round"
		stroke-dasharray={deco.dashed ? '2 6' : '0'}
		class="transition-opacity group-hover/edge:opacity-70 {deco.pulsing
			? 'motion-safe:animate-pulse'
			: ''}"
		data-on-route={onRoute}
	/>
	<!-- The curve stops short of the node on its rail; an elbow already lands on
	     the node's edge. -->
	{#if !elbow}
		<circle cx={geometry.from.x} cy={geometry.from.y} r="4" fill={stroke} />
		<circle cx={geometry.to.x} cy={geometry.to.y} r="4" fill={stroke} />
	{/if}
	{#if deco.badgeCount > 0}
		<foreignObject
			x={geometry.center.x - badgeWidth / 2}
			y={geometry.center.y - 10}
			width={badgeWidth}
			height="20"
			class="pointer-events-none"
		>
			<div
				class="flex h-full items-center justify-center gap-0.5 rounded-full border border-neutral-300 bg-white px-1 dark:border-neutral-700 dark:bg-neutral-900"
			>
				{#if c.kind === 'stargate'}
					<OrbitIcon class="size-3.5" style="color: #0ea5e9" />
				{/if}
				{#if deco.sizeLabel}
					<span class="text-[13px] leading-none font-bold text-neutral-500">
						{deco.sizeLabel}
					</span>
				{/if}
				{#if deco.massColor}
					<WeightIcon class="size-3.5" style="color: {deco.massColor}" />
				{/if}
				{#if deco.timeColor}
					<ClockIcon
						class="size-3.5 {deco.pulsing ? 'motion-safe:animate-pulse' : ''}"
						style="color: {deco.timeColor}"
					/>
					{#if timeLeft}
						<span
							class="text-[10px] leading-none font-semibold tabular-nums"
							style="color: {deco.timeColor}"
							data-testid="edge-time-left">{timeLeft}</span
						>
					{/if}
				{/if}
			</div>
		</foreignObject>
	{/if}
	<!-- Wide invisible hit area, drawn last so it sits on top. -->
	<path
		d={geometry.d}
		fill="none"
		stroke="transparent"
		stroke-width="24"
		style="cursor:pointer"
		role="presentation"
		data-testid="edge-hit"
		data-connection-id={c.id}
		onpointerdown={(ev) => ev.stopPropagation()}
		onclick={(ev) => {
			ev.stopPropagation();
			map.closeMenu();
			map.connectionPopover = { id: c.id, x: ev.clientX, y: ev.clientY };
		}}
		oncontextmenu={(ev) => {
			ev.preventDefault();
			ev.stopPropagation();
			map.connectionPopover = null;
			map.openMenu(ev.clientX, ev.clientY, { kind: 'connection', id: c.id });
		}}
	/>
</g>
