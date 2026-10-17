<script lang="ts">
	// The A→B planner: two pickers with on-map suggestions, the pinned route, and the
	// ignored-systems escape hatch.
	import ArrowLeftRightIcon from '@lucide/svelte/icons/arrow-left-right';
	import XIcon from '@lucide/svelte/icons/x';

	import { Button } from '$lib/components/ui/button';
	import RouteList from '../RouteList.svelte';
	import SystemCombobox from '../../pickers/SystemCombobox.svelte';
	import { jumpTone as badgeTone, type RouteGraph } from '$lib/routing/algorithm';
	import type { MapState } from '../../../state/map-state.svelte';
	import { pickerSuggestions } from './suggestions';

	let {
		map,
		graph,
		originOnly = false,
	}: {
		map: MapState;
		graph: RouteGraph | null;
		/** Just the origin, for Find: it has no destination, so no route to show. */
		originOnly?: boolean;
	} = $props();

	const abResult = $derived.by(() => {
		if (!graph || map.route.fromId === null || map.route.toId === null) return null;
		// From is also the shared origin whenever it is set, so this is the search the
		// watchlist and Find already ran.
		return map.route.search(map.route.fromId)?.routeTo(map.route.toId) ?? null;
	});
	const abPath = $derived(abResult?.route.map((s) => s.id) ?? []);
	// A hovered row anywhere on the page temporarily overrides the pinned A→B highlight.
	$effect(() => {
		map.route.path = map.route.hoverPath ?? abPath;
	});

	const suggestions = $derived(pickerSuggestions(map));

	function swap() {
		[map.route.fromId, map.route.toId] = [map.route.toId, map.route.fromId];
	}
</script>

<div class="flex flex-col gap-2 border-b border-border/50 p-3 text-xs">
	<div class="flex items-center gap-1.5">
		<SystemCombobox
			placeholder="Origin"
			value={map.route.fromId}
			{suggestions}
			onpick={(id) => (map.route.fromId = id)}
		/>
		{#if !originOnly}
			<Button variant="ghost" size="icon-xs" aria-label="Swap" onclick={swap}>
				<ArrowLeftRightIcon />
			</Button>
			<SystemCombobox
				placeholder="Destination"
				value={map.route.toId}
				{suggestions}
				onpick={(id) => (map.route.toId = id)}
			/>
		{/if}
	</div>

	{#if originOnly}
		<!-- No route on the Find tab. -->
	{:else if abResult === null && map.route.fromId !== null && map.route.toId !== null}
		<p class="text-muted-foreground" data-testid="no-route">No route found</p>
	{:else if abResult}
		<div class="flex items-center justify-between font-medium">
			<span class={badgeTone(abResult.jumps)} data-testid="route-jumps">{abResult.jumps} jumps</span
			>
			<span class="flex items-center gap-2">
				{#if map.route.ignoredSystems.size > 0}
					<button
						class="text-[11px] text-muted-foreground underline-offset-2 hover:underline"
						data-testid="clear-ignored"
						onclick={() => map.route.clearIgnored()}
					>
						{map.route.ignoredSystems.size} ignored · Clear
					</button>
				{/if}
				<Button
					variant="ghost"
					size="icon-xs"
					aria-label="Clear route"
					onclick={() => {
						map.route.fromId = null;
						map.route.toId = null;
					}}
				>
					<XIcon />
				</Button>
			</span>
		</div>
		<RouteList
			steps={map.route.withSignatures(abResult.route)}
			onignore={(id) => map.route.ignoreSystem(id)}
		/>
	{:else if map.route.ignoredSystems.size > 0}
		<button
			class="self-start text-[11px] text-muted-foreground underline-offset-2 hover:underline"
			data-testid="clear-ignored"
			onclick={() => map.route.clearIgnored()}
		>
			{map.route.ignoredSystems.size} ignored · Clear
		</button>
	{/if}
</div>
