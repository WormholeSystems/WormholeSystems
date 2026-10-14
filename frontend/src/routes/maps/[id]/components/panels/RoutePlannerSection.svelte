<script lang="ts">
	// The A→B planner: two pickers with on-map suggestions, the pinned route, and the
	// ignored-systems escape hatch.
	import ArrowLeftRightIcon from '@lucide/svelte/icons/arrow-left-right';
	import XIcon from '@lucide/svelte/icons/x';

	import type { SystemSearchResult } from '$lib/api/types/SystemSearchResult';
	import { Button } from '$lib/components/ui/button';
	import RouteList from './RouteList.svelte';
	import SystemCombobox from '../pickers/SystemCombobox.svelte';
	import { jumpTone as badgeTone, type RouteGraph } from '$lib/routing/algorithm';
	import type { MapState } from '../../state/map-state.svelte';

	let { map, graph }: { map: MapState; graph: RouteGraph | null } = $props();

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

	// Both pickers offer the systems already in play before anything is typed. They sit inside
	// the picker, so choosing one is unambiguous about which end it fills.
	const suggestedIds = $derived.by(() => {
		const picks: { id: number; reason: string; icon: 'selected' | 'location' | 'pinned' }[] = [];
		const active = map.activeSystem;
		if (active?.kind === 'system') {
			picks.push({ id: active.solar_system_id, reason: 'Selected system', icon: 'selected' });
		}
		const character = map.characters.mine.find((c) => c.online && c.solar_system_id !== null);
		if (
			character?.solar_system_id != null &&
			!picks.some((p) => p.id === character.solar_system_id)
		) {
			picks.push({ id: character.solar_system_id, reason: 'Where you are', icon: 'location' });
		}
		for (const entry of map.watchlist.all.filter((w) => w.is_pinned).slice(0, 5)) {
			if (picks.some((p) => p.id === entry.solar_system_id)) continue;
			picks.push({ id: entry.solar_system_id, reason: 'Pinned on the watchlist', icon: 'pinned' });
		}
		return picks;
	});
	// Only those we can render as a proper row; an unresolved one would show as a bare id.
	const suggestions = $derived(
		suggestedIds
			.map((p) => ({ system: map.systemInfo(p.id), reason: p.reason, icon: p.icon }))
			.filter(
				(p): p is { system: SystemSearchResult; reason: string; icon: typeof p.icon } =>
					p.system !== null,
			),
	);

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
		<Button variant="ghost" size="icon-xs" aria-label="Swap" onclick={swap}>
			<ArrowLeftRightIcon />
		</Button>
		<SystemCombobox
			placeholder="Destination"
			value={map.route.toId}
			{suggestions}
			onpick={(id) => (map.route.toId = id)}
		/>
	</div>

	{#if abResult === null && map.route.fromId !== null && map.route.toId !== null}
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
