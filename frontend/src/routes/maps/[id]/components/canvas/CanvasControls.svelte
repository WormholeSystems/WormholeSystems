<script lang="ts">
	// The canvas's corner controls: placement mode on the left, zoom on the right.
	import WaypointsIcon from '@lucide/svelte/icons/waypoints';
	import WorkflowIcon from '@lucide/svelte/icons/workflow';

	import type { MapState } from '../../state/map-state.svelte';
	import BackgroundPopover from './BackgroundPopover.svelte';
	import Hint from '$lib/components/Hint.svelte';

	let { map }: { map: MapState } = $props();
</script>

<!-- Picking the map's own mode clears the override, so a later change to the map still
     reaches this viewer. -->
{#if map.data?.map.allow_layout_override}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="absolute bottom-3 left-3 flex items-center overflow-hidden border border-border bg-card"
		data-testid="placement-controls"
		onpointerdown={(ev) => ev.stopPropagation()}
	>
		{#each [{ mode: 'manual', label: 'Custom placement', icon: WaypointsIcon }, { mode: 'tree', label: 'Automatic placement', icon: WorkflowIcon }] as option (option.mode)}
			{@const Icon = option.icon}
			<Hint text={option.label}>
				{#snippet child({ props })}
					<button
						class="px-2 py-1 {map.layout === option.mode
							? 'bg-accent text-foreground'
							: 'text-muted-foreground hover:bg-accent/50 hover:text-foreground'}"
						aria-label={option.label}
						{...props}
						aria-pressed={map.layout === option.mode}
						data-testid="placement-{option.mode}"
						onclick={() => map.placement.setForMe(option.mode as 'manual' | 'tree')}
					>
						<Icon class="size-4" />
					</button>
				{/snippet}
			</Hint>
		{/each}
	</div>
{/if}

<!-- The press is stopped here, like the scrollbars do: the canvas captures the pointer on
     background press, which retargets the click onto the canvas and never reaches the
     button. -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="absolute right-3 bottom-3 flex items-center overflow-hidden border border-border bg-card"
	data-testid="zoom-controls"
	onpointerdown={(ev) => ev.stopPropagation()}
>
	<BackgroundPopover {map} />
	<button
		class="border-l border-border px-2.5 py-1 text-sm text-muted-foreground hover:bg-accent hover:text-foreground"
		aria-label="Zoom out"
		onclick={() => map.camera.zoomBy(-1)}
	>
		−
	</button>
	<span
		class="border-x border-border px-2 py-1 text-xs tabular-nums text-muted-foreground"
		data-testid="zoom-level"
	>
		{Math.round(map.camera.zoom * 100)}%
	</span>
	<button
		class="px-2.5 py-1 text-sm text-muted-foreground hover:bg-accent hover:text-foreground"
		aria-label="Zoom in"
		onclick={() => map.camera.zoomBy(1)}
	>
		+
	</button>
</div>
