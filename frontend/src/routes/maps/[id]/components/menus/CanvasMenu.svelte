<script lang="ts">
	// The right-click menu on the canvas: a bits-ui menu anchored at the point that was
	// clicked rather than at a trigger, since the canvas, its nodes and its edges each open
	// it with a different target. What it holds follows what was clicked.
	//
	// The open state stays in `map.menu`, which the canvas gestures already own; bits-ui only
	// renders it, and says when it should close.
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import type { MapState, Menu } from '../../state/map-state.svelte';
	import ConnectionMenu from './ConnectionMenu.svelte';
	import MapMenu from './MapMenu.svelte';
	import NodeMenu from './NodeMenu.svelte';

	let { map, menu }: { map: MapState; menu: Menu } = $props();

	// A zero-size box at the click, which is all the positioning needs to measure.
	const anchor = $derived({
		getBoundingClientRect: () => new DOMRect(menu.x, menu.y, 0, 0),
	});

	const connection = $derived(
		menu.target.kind === 'connection' ? (map.connections.byId.get(menu.target.id) ?? null) : null,
	);

	let content = $state<HTMLElement | null>(null);

	function onOpenChange(open: boolean) {
		// A right-click elsewhere opens the next menu before this one hears it was clicked
		// outside of; only the menu still showing may close.
		if (!open && map.menu === menu) map.closeMenu();
	}
</script>

<DropdownMenu.Root open {onOpenChange}>
	<DropdownMenu.Content
		customAnchor={anchor}
		side="bottom"
		align="start"
		sideOffset={0}
		preventScroll={false}
		class="w-52"
		data-testid="context-menu"
		bind:ref={content}
		onOpenAutoFocus={(ev) => {
			// Focused without scrolling: the default focus can scroll the page to the menu
			// before it is positioned, which drags the canvas out from under the click.
			ev.preventDefault();
			content?.focus({ preventScroll: true });
		}}
		onCloseAutoFocus={(ev) => ev.preventDefault()}
		oncontextmenu={(ev: MouseEvent) => ev.preventDefault()}
	>
		{#if menu.target.kind === 'map'}
			<MapMenu {map} {menu} />
		{:else if menu.target.kind === 'node'}
			<NodeMenu {map} system={menu.target.system} />
		{:else if connection}
			<ConnectionMenu {map} {connection} />
		{/if}
	</DropdownMenu.Content>
</DropdownMenu.Root>
