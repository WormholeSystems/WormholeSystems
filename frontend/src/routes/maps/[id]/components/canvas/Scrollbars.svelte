<script lang="ts">
	// Proportional scrollbars: thumb size and position are the viewport-over-world ratio at
	// the current zoom. They fade out once the view settles, since they only answer "where am
	// I in the chain" while you are navigating.
	import { clamp } from '$lib/map/helpers';
	import type { MapState } from '../../state/map-state.svelte';

	let { map }: { map: MapState } = $props();

	let hTrack: HTMLElement | null = null;
	let vTrack: HTMLElement | null = null;
	let hDragging = false;
	let vDragging = false;

	// Visible world span = viewport_size / zoom. Thumb fraction = visible / world. The size
	// is the observed one: measuring the element here would force a layout every pan frame.
	const hThumb = $derived.by(() => {
		const frac = Math.min(
			map.camera.viewportSize.width / map.camera.zoom / map.grid.world_width,
			1,
		);
		const start = clamp(-map.camera.pan.x / map.camera.zoom / map.grid.world_width, 0, 1 - frac);
		return { start: start * 100, size: frac * 100 };
	});
	const vThumb = $derived.by(() => {
		const frac = Math.min(
			map.camera.viewportSize.height / map.camera.zoom / map.grid.world_height,
			1,
		);
		const start = clamp(-map.camera.pan.y / map.camera.zoom / map.grid.world_height, 0, 1 - frac);
		return { start: start * 100, size: frac * 100 };
	});

	function hSet(clientX: number) {
		if (!hTrack) return;
		const r = hTrack.getBoundingClientRect();
		if (r.width <= 0) return;
		const cf = clamp((clientX - r.left) / r.width, 0, 1);
		const frac = hThumb.size / 100;
		const start = clamp(cf - frac / 2, 0, 1 - frac);
		map.camera.pan = { ...map.camera.pan, x: -start * map.camera.zoom * map.grid.world_width };
		map.camera.wakeScrollbars();
	}

	function vSet(clientY: number) {
		if (!vTrack) return;
		const r = vTrack.getBoundingClientRect();
		if (r.height <= 0) return;
		const cf = clamp((clientY - r.top) / r.height, 0, 1);
		const frac = vThumb.size / 100;
		const start = clamp(cf - frac / 2, 0, 1 - frac);
		map.camera.pan = { ...map.camera.pan, y: -start * map.camera.zoom * map.grid.world_height };
		map.camera.wakeScrollbars();
	}

	const thumb = 'absolute rounded-full bg-muted-foreground/50';
	// The hit area goes with the thumb: a track that still catches clicks while invisible
	// would be misleading.
	const track = $derived(
		map.camera.scrollbarsVisible ? 'opacity-100' : 'pointer-events-none opacity-0',
	);
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	bind:this={hTrack}
	class="absolute inset-x-1 bottom-1 h-2 cursor-pointer transition-opacity duration-300 {track}"
	onpointerdown={(ev) => {
		ev.stopPropagation();
		hDragging = true;
		hTrack?.setPointerCapture(ev.pointerId);
		hSet(ev.clientX);
	}}
	onpointermove={(ev) => hDragging && hSet(ev.clientX)}
	onpointerup={() => (hDragging = false)}
>
	<div
		class={thumb}
		style="top:2px;bottom:2px;min-width:30px"
		style:left="{hThumb.start}%"
		style:width="{hThumb.size}%"
	></div>
</div>
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	bind:this={vTrack}
	class="absolute inset-y-1 right-1 w-2 cursor-pointer transition-opacity duration-300 {track}"
	onpointerdown={(ev) => {
		ev.stopPropagation();
		vDragging = true;
		vTrack?.setPointerCapture(ev.pointerId);
		vSet(ev.clientY);
	}}
	onpointermove={(ev) => vDragging && vSet(ev.clientY)}
	onpointerup={() => (vDragging = false)}
>
	<div
		class={thumb}
		style="left:2px;right:2px;min-height:30px"
		style:top="{vThumb.start}%"
		style:height="{vThumb.size}%"
	></div>
</div>
