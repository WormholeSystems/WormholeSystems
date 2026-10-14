<script lang="ts">
	// The shared watchlist, jump counts from the shared origin, sortable by column.
	import PinIcon from '@lucide/svelte/icons/pin';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';

	import type { WatchlistEntry } from '$lib/api/types/WatchlistEntry';
	import SortHeader from '$lib/components/SortHeader.svelte';
	import SystemMenu from '../../system-menu/SystemMenu.svelte';
	import SystemRow from '../../pickers/SystemRow.svelte';
	import { SYSTEM_CELLS_4, SYSTEM_LIST_ACTIONS, SYSTEM_ROW } from '../../pickers/columns';
	import { compareWatchlistEntries } from '$lib/map/watchlist';
	import { jumpTone as badgeTone, type RouteGraph, type RouteResult } from '$lib/routing/algorithm';
	import { sortState } from '$lib/sort-state.svelte';
	import type { MapState } from '../../../state/map-state.svelte';
	import RoutePopover from '../RoutePopover.svelte';

	let { map, graph, origin }: { map: MapState; graph: RouteGraph | null; origin: number | null } =
		$props();

	const canWrite = $derived(map.canWrite);
	const originName = $derived(origin === null ? null : (map.systemInfo(origin)?.name ?? '…'));
	const watchRoutes = $derived.by(() => {
		if (!graph || origin === null) return new Map<number, RouteResult>();
		const search = map.route.search(origin);
		return (
			search?.routesTo(map.watchlist.all.map((w) => w.solar_system_id)) ??
			new Map<number, RouteResult>()
		);
	});

	const SORT_COLUMNS = ['system', 'region', 'jumps'] as const;
	const sort = sortState('watchlist-sort', SORT_COLUMNS, { column: 'system', direction: 'asc' });

	function jumpsOf(entry: WatchlistEntry): number | null {
		return watchRoutes.get(entry.solar_system_id)?.jumps ?? null;
	}

	const sortedWatchlist = $derived.by(() => {
		const dir = sort.current.direction === 'asc' ? 1 : -1;
		return map.watchlist.all.toSorted(
			(a, b) =>
				compareWatchlistEntries(a, b, sort.current.column, (id) => map.systemInfo(id), jumpsOf) *
				dir,
		);
	});
</script>

<!-- One grid owns the tracks so every row and the header share column widths. -->
<div class={SYSTEM_LIST_ACTIONS}>
	<div
		class="{SYSTEM_ROW} border-b border-border/30 bg-muted/20 px-3 py-1.5 font-mono text-[10px] tracking-wider text-muted-foreground uppercase"
	>
		<SortHeader column="system" sort={sort.current} onsort={sort.toggle} class="col-span-2 min-w-0">
			<span class="truncate">
				Watchlist{#if originName}&nbsp;· from {originName}{/if}
			</span>
		</SortHeader>
		<SortHeader column="region" sort={sort.current} onsort={sort.toggle} class="min-w-0">
			Region
		</SortHeader>
		<span></span>
		<SortHeader column="jumps" sort={sort.current} onsort={sort.toggle} class="justify-end">
			J
		</SortHeader>
		<span></span>
	</div>

	{#if sortedWatchlist.length === 0}
		<p
			class="col-span-full p-3 text-center font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase"
		>
			Watchlist empty
		</p>
	{/if}
	{#each sortedWatchlist as entry (entry.id)}
		{@const r = map.systemInfo(entry.solar_system_id)}
		{@const route = watchRoutes.get(entry.solar_system_id)}
		<div
			class="{SYSTEM_ROW} border-b border-border/30 px-3 py-1 text-xs hover:bg-muted/30"
			data-testid="watchlist-row"
			role="listitem"
			onmouseenter={() => (map.route.hoverPath = route?.route.map((s) => s.id) ?? null)}
			onmouseleave={() => (map.route.hoverPath = null)}
		>
			{#if r}
				<SystemMenu system={r} class={SYSTEM_CELLS_4}>
					<SystemRow system={r} />
				</SystemMenu>
			{:else}
				<span class="col-span-4 truncate text-muted-foreground">
					{entry.solar_system_id}
				</span>
			{/if}
			{#if route}
				<RoutePopover {map} steps={route.route}>
					<span
						class="cursor-pointer text-xs font-medium {badgeTone(route.jumps)}"
						data-testid="route-jumps-badge"
					>
						{route.jumps}j
					</span>
				</RoutePopover>
			{:else}
				<!-- `nowrap` because a hyphen is a line-break opportunity: with every row
				     unreachable, the dashes wrap and every row grows a line. -->
				<span class="text-[10px] whitespace-nowrap text-muted-foreground/60"> -- </span>
			{/if}
			{#if canWrite}
				<span class="flex items-center justify-end gap-1">
					<button
						class="text-muted-foreground hover:text-foreground {entry.is_pinned
							? 'text-amber-400 hover:text-amber-400'
							: ''}"
						title={entry.is_pinned ? 'Unpin' : 'Pin as quick-pick'}
						aria-label="Pin {r?.name ?? entry.solar_system_id}"
						onclick={() => map.watchlist.setPinned(entry.id, !entry.is_pinned)}
					>
						<PinIcon class="size-3" />
					</button>
					<button
						class="text-muted-foreground hover:text-destructive"
						title="Remove from watchlist"
						aria-label="Remove {r?.name ?? entry.solar_system_id}"
						onclick={() => map.watchlist.remove(entry.id)}
					>
						<Trash2Icon class="size-3" />
					</button>
				</span>
			{:else}
				<span></span>
			{/if}
		</div>
	{/each}
</div>
