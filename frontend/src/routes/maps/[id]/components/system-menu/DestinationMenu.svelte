<script lang="ts">
	// A minimal right-click menu for any ESI destination that is not a solar system
	// (stations, structures): set destination / add waypoint per online character.
	// Solar systems use the richer SystemMenu instead.
	import type { Snippet } from 'svelte';
	import { getMapContext } from './context';
	import WaypointSubmenus from './WaypointSubmenus.svelte';

	import * as ContextMenu from '$lib/components/ui/context-menu';

	let {
		destinationId,
		class: cls = 'contents',
		children,
	}: {
		/** An ESI destination id: station, structure, or system. */
		destinationId: number;
		class?: string;
		children: Snippet;
	} = $props();

	const getMap = getMapContext();
	const map = $derived(getMap?.());
</script>

<ContextMenu.Root>
	<ContextMenu.Trigger class={cls} data-testid="destination-menu-trigger">
		{@render children()}
	</ContextMenu.Trigger>
	<ContextMenu.Content class="w-52" data-testid="destination-menu">
		{#if map !== undefined}
			<WaypointSubmenus {map} {destinationId} />
		{:else}
			<ContextMenu.Item disabled>No map context</ContextMenu.Item>
		{/if}
	</ContextMenu.Content>
</ContextMenu.Root>
