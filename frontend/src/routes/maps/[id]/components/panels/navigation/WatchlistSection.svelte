<script lang="ts">
	// A watchlist, the map's shared one or the user's own, with jump counts from the shared
	// origin, sortable by column.
	import EllipsisVerticalIcon from '@lucide/svelte/icons/ellipsis-vertical';
	import PinIcon from '@lucide/svelte/icons/pin';
	import PinOffIcon from '@lucide/svelte/icons/pin-off';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';

	import type { WatchlistEntry } from '$lib/api/types/WatchlistEntry';
	import SortHeader from '$lib/components/SortHeader.svelte';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import SystemMenu from '../../system-menu/SystemMenu.svelte';
	import SystemRow from '../../pickers/SystemRow.svelte';
	import { SYSTEM_CELLS_4, SYSTEM_LIST_ACTIONS, SYSTEM_ROW } from '../../pickers/columns';
	import { compareWatchlistEntries } from '$lib/map/watchlist';
	import { jumpTone as badgeTone, type RouteGraph, type RouteResult } from '$lib/routing/algorithm';
	import { sortState } from '$lib/sort-state.svelte';
	import { untrack } from 'svelte';
	import type { MapState } from '../../../state/map-state.svelte';
	import type { WatchlistList } from '../../../state/watchlist';
	import RoutePopover from '../RoutePopover.svelte';
	import Hint from '$lib/components/Hint.svelte';

	let {
		map,
		list,
		canEdit,
		empty,
		graph,
		origin,
	}: {
		map: MapState;
		list: WatchlistList;
		canEdit: boolean;
		/** What the list says when there is nothing on it. */
		empty: string;
		graph: RouteGraph | null;
		origin: number | null;
	} = $props();

	const watchRoutes = $derived.by(() => {
		if (!graph || origin === null) return new Map<number, RouteResult>();
		const search = map.route.search(origin);
		return (
			search?.routesTo(list.all.map((w) => w.solar_system_id)) ?? new Map<number, RouteResult>()
		);
	});

	const SORT_COLUMNS = ['system', 'region', 'jumps'] as const;
	const sort = sortState('watchlist-sort', SORT_COLUMNS, { column: 'system', direction: 'asc' });

	function jumpsOf(entry: WatchlistEntry): number | null {
		return watchRoutes.get(entry.solar_system_id)?.jumps ?? null;
	}

	// Rows on their way out: faded and struck through until the list no longer has them, so a
	// removal is seen to be happening rather than the row just vanishing a moment later.
	// An undone removal comes back under the same id, so a mark is dropped once its write
	// settles and the list has caught up, not kept for the life of the panel.
	let removing = $state(new Set<number>());
	function unmark(entryId: number) {
		const next = new Set(removing);
		next.delete(entryId);
		removing = next;
	}
	function remove(entryId: number) {
		removing = new Set(removing).add(entryId);
		list.remove(entryId).catch(() => unmark(entryId));
	}
	$effect(() => {
		const present = new Set(list.all.map((e) => e.id));
		for (const id of removing) if (!present.has(id)) untrack(() => unmark(id));
	});

	const sortedWatchlist = $derived.by(() => {
		const dir = sort.current.direction === 'asc' ? 1 : -1;
		return [...list.all].sort(
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
			<span class="truncate">System</span>
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
			{empty}
		</p>
	{/if}
	{#each sortedWatchlist as entry (entry.id)}
		{@const r = map.systemInfo(entry.solar_system_id)}
		{@const route = watchRoutes.get(entry.solar_system_id)}
		{@const name = r?.name ?? String(entry.solar_system_id)}
		<div
			class="{SYSTEM_ROW} border-b border-border/30 px-3 py-1 text-xs transition-opacity hover:bg-muted/30 {removing.has(
				entry.id,
			)
				? 'pointer-events-none line-through opacity-40'
				: ''}"
			data-testid="watchlist-row"
			data-removing={removing.has(entry.id) || undefined}
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
			<span class="flex items-center justify-end gap-1">
				<!-- Pinned is a state, so it shows on the row; changing it lives in the menu. -->
				{#if entry.is_pinned}
					<Hint text="Pinned: offered in the route pickers" focusable={false} label>
						{#snippet child({ props })}
							<span
								class="flex size-3 items-center justify-center"
								{...props}
								data-testid="watchlist-pinned"
							>
								<PinIcon class="size-3 fill-amber-400 text-amber-400" />
							</span>
						{/snippet}
					</Hint>
				{:else}
					<span class="size-3"></span>
				{/if}
				{#if canEdit}
					<DropdownMenu.Root>
						<Hint text={'Watchlist actions'}>
							{#snippet child({ props })}
								<DropdownMenu.Trigger
									class="flex items-center text-muted-foreground hover:text-foreground disabled:opacity-40"
									{...props}
									aria-label="Actions for {name}"
									disabled={removing.has(entry.id)}
									data-testid="watchlist-row-menu"
								>
									<EllipsisVerticalIcon class="size-3.5" />
								</DropdownMenu.Trigger>
							{/snippet}
						</Hint>
						<DropdownMenu.Content align="end" class="min-w-44">
							<DropdownMenu.Item
								class="text-xs"
								onclick={() => list.setPinned(entry.id, !entry.is_pinned)}
							>
								{#if entry.is_pinned}
									<PinOffIcon class="size-3.5" />
									Unpin
								{:else}
									<PinIcon class="size-3.5" />
									Pin as a route quick-pick
								{/if}
							</DropdownMenu.Item>
							<DropdownMenu.Separator />
							<DropdownMenu.Item
								variant="destructive"
								class="text-xs"
								onclick={() => remove(entry.id)}
							>
								<Trash2Icon class="size-3.5" />
								Remove from the watchlist
							</DropdownMenu.Item>
						</DropdownMenu.Content>
					</DropdownMenu.Root>
				{:else}
					<span class="size-3.5"></span>
				{/if}
			</span>
		</div>
	{/each}
</div>
