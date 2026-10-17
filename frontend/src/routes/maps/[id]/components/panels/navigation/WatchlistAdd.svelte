<script lang="ts">
	// The header's "+": search any system and put it on the watchlist the card is showing.
	import PlusIcon from '@lucide/svelte/icons/plus';

	import { q } from '$lib/api/queries';
	import * as Command from '$lib/components/ui/command';
	import * as Popover from '$lib/components/ui/popover';
	import SystemMenu from '../../system-menu/SystemMenu.svelte';
	import SystemRow from '../../pickers/SystemRow.svelte';
	import { SYSTEM_CELLS_4, SYSTEM_LIST, SYSTEM_ROW } from '../../pickers/columns';
	import { searchQuery } from '$lib/search-query.svelte';
	import type { WatchlistList } from '../../../state/watchlist';
	import Hint from '$lib/components/Hint.svelte';

	let {
		list,
		personal = false,
		disabled = null,
	}: {
		list: WatchlistList;
		personal?: boolean;
		/** Why it cannot be used here. It stays in place regardless, so the header holds still. */
		disabled?: string | null;
	} = $props();
	const label = $derived(
		disabled ?? (personal ? 'Add to your watchlist' : 'Add to the map watchlist'),
	);

	let addOpen = $state(false);
	let addQuery = $state('');
	const addSearch = searchQuery({
		term: () => addQuery,
		query: (settled) => q.searchSystems(settled),
		enabled: () => addOpen,
		minChars: 1,
	});
	const addResults = $derived(addSearch.results);
	function addToWatchlist(id: number) {
		list.add(id);
		addOpen = false;
	}
</script>

<Popover.Root bind:open={addOpen}>
	<Hint text={label}>
		{#snippet child({ props })}
			<Popover.Trigger
				class="text-muted-foreground transition-colors hover:text-foreground disabled:cursor-default disabled:opacity-40 disabled:hover:text-muted-foreground"
				disabled={disabled !== null}
				{...props}
				aria-label={label}
				data-testid="watchlist-add"
			>
				<PlusIcon class="size-4" />
			</Popover.Trigger>
		{/snippet}
	</Hint>
	<Popover.Content class="w-96 p-0" align="end">
		<Command.Root shouldFilter={false}>
			<Command.Input placeholder="Watch a system…" bind:value={addQuery} />
			<!-- The list owns the tracks and rows are subgrids of them. The trailing
			     track is Command's own check indicator, appended to every item. -->
			<Command.List class="max-h-48 p-1 {SYSTEM_LIST}">
				<Command.Empty class="col-span-full">
					{addQuery.trim().length < 2
						? 'Type at least two characters to search.'
						: 'No systems found.'}
				</Command.Empty>
				{#each addResults as s (s.id)}
					<Command.Item
						value={String(s.id)}
						onSelect={() => addToWatchlist(s.id)}
						class={SYSTEM_ROW}
					>
						<SystemMenu system={s} class={SYSTEM_CELLS_4}>
							<SystemRow system={s} />
						</SystemMenu>
					</Command.Item>
				{/each}
			</Command.List>
		</Command.Root>
	</Popover.Content>
</Popover.Root>
