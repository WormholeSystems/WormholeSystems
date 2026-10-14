<script lang="ts">
	// The solar-system context menu for any rendered system reference: wrap it and
	// right-click (or long-press) it. The map node's menu offers the same items plus its own.
	import type { Snippet } from 'svelte';

	import type { SystemSearchResult } from '$lib/api/types/SystemSearchResult';
	import * as ContextMenu from '$lib/components/ui/context-menu';
	import { getMapContext } from './context';
	import SystemMenuItems from './SystemMenuItems.svelte';

	let {
		system,
		class: cls = 'contents',
		children,
	}: {
		system: SystemSearchResult;
		/** Trigger element class; `contents` keeps the wrapped fragments in their row layout. */
		class?: string;
		children: Snippet;
	} = $props();

	// A getter, so the context value registered at init survives the page swapping maps.
	const getMap = getMapContext();
	const map = $derived(getMap());
</script>

<ContextMenu.Root>
	<ContextMenu.Trigger class={cls} data-testid="system-menu-trigger" data-system-id={system.id}>
		{@render children()}
	</ContextMenu.Trigger>
	<ContextMenu.Content class="w-52" data-testid="system-menu">
		<SystemMenuItems {map} {system} />
	</ContextMenu.Content>
</ContextMenu.Root>
