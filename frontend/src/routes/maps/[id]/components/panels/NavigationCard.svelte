<script lang="ts">
	// Three tabs in the header: the map's shared watchlist, the user's own, and the
	// closest-systems Find. The route planner sits under the header on every tab, so nobody
	// has to go looking for it; Find keeps just its origin. All three measure from that one
	// origin: the route From, else the active system, else the tracked character.
	import ListIcon from '@lucide/svelte/icons/list';
	import SearchIcon from '@lucide/svelte/icons/search';
	import UserIcon from '@lucide/svelte/icons/user';
	import * as v from 'valibot';

	import MapPanel from '$lib/components/map-panel/MapPanel.svelte';
	import MapPanelContent from '$lib/components/map-panel/MapPanelContent.svelte';
	import MapPanelHeader from '$lib/components/map-panel/MapPanelHeader.svelte';
	import { Separator } from '$lib/components/ui/separator';
	import * as Tabs from '$lib/components/ui/tabs';
	import { readStored, writeStored } from '$lib/storage';
	import type { MapState } from '../../state/map-state.svelte';
	import FindSection from './navigation/FindSection.svelte';
	import RoutePlannerSection from './navigation/RoutePlannerSection.svelte';
	import RouteSettings from './navigation/RouteSettings.svelte';
	import WatchlistAdd from './navigation/WatchlistAdd.svelte';
	import WatchlistSection from './navigation/WatchlistSection.svelte';
	import RouteOriginBadge from './RouteOriginBadge.svelte';
	import Hint from '$lib/components/Hint.svelte';

	let { map }: { map: MapState } = $props();

	const PREF_LABELS = {
		shorter: 'Shortest',
		safer: 'Safer',
		less_secure: 'Less Secure',
	} satisfies Record<string, string>;

	const TAB_KEYS = ['watchlist', 'personal', 'find'] as const;
	type Tab = (typeof TAB_KEYS)[number];
	const TAB_KEY = 'navigation-tab';
	let tab = $state<Tab>(readStored(TAB_KEY, v.picklist(TAB_KEYS), 'watchlist'));
	// A watcher on a share link has no list of their own.
	// `short` is what a narrow card has room for; Find is only ever its icon.
	const tabs = $derived([
		{
			key: 'watchlist' as const,
			label: 'Watchlist',
			short: 'Map',
			count: map.watchlist.all.length,
		},
		...(map.signedIn
			? [
					{
						key: 'personal' as const,
						label: 'Personal',
						short: 'Mine',
						count: map.personalWatchlist.all.length,
					},
				]
			: []),
		{ key: 'find' as const, label: 'Find', short: 'Find', count: 0 },
	]);
	const shown = $derived(tabs.some((t) => t.key === tab) ? tab : 'watchlist');

	const graph = $derived(map.route.graph);
	const origin = $derived(map.routeOrigin);
</script>

<MapPanel testid="navigation-card">
	<MapPanelHeader>
		<!-- The header gives way from the right as the card narrows: the preference, then the
		     origin, then the tab names shorten, then the title goes. -->
		<span class="hidden truncate @[16rem]/panel-header:inline">Navigation</span>
		<span class="ml-1 hidden text-muted-foreground/60 normal-case @[28rem]/panel-header:inline">
			{PREF_LABELS[map.userSettings?.route_preference ?? 'shorter']}
		</span>
		{#snippet actions()}
			<span class="hidden min-w-0 @[24rem]/panel-header:flex">
				<RouteOriginBadge {map} />
			</span>
			<!-- The same tabs as the skyhooks card's lava and ice. -->
			<Tabs.Root
				value={shown}
				onValueChange={(next) => {
					if (!next) return;
					tab = next as Tab;
					writeStored(TAB_KEY, tab);
				}}
				class="w-fit"
				data-testid="navigation-tabs"
			>
				<Tabs.List variant="line" class="h-6">
					{#each tabs as t (t.key)}
						{#snippet trigger(props: Record<string, unknown>)}
							<Tabs.Trigger
								value={t.key}
								class="px-1 text-[10px]"
								aria-label={t.label}
								{...props}
								data-testid="navigation-tab-{t.key}"
							>
								{#if t.key === 'find'}
									<SearchIcon class="size-3.5" />
								{:else}
									<span class="hidden @[20rem]/panel-header:inline">{t.label}</span>
									<span class="hidden @[12rem]/panel-header:inline @[20rem]/panel-header:hidden"
										>{t.short}</span
									>
									{#if t.key === 'personal'}
										<UserIcon class="size-3.5 @[12rem]/panel-header:hidden" />
									{:else}
										<ListIcon class="size-3.5 @[12rem]/panel-header:hidden" />
									{/if}
								{/if}
								{#if t.count > 0}
									<span class="hidden font-mono text-amber-400 @[12rem]/panel-header:inline"
										>{t.count}</span
									>
								{/if}
							</Tabs.Trigger>
						{/snippet}
						<!-- Only Find is an icon alone; the others say what they are. -->
						{#if t.key === 'find'}
							<Hint text="Find the closest systems">
								{#snippet child({ props })}
									{@render trigger(props)}
								{/snippet}
							</Hint>
						{:else}
							{@render trigger({})}
						{/if}
					{/each}
				</Tabs.List>
			</Tabs.Root>
			<Separator
				orientation="vertical"
				class="mx-0.5 hidden data-[orientation=vertical]:h-4 @[14rem]/panel-header:block"
			/>
			<RouteSettings {map} />
			{#if shown === 'personal'}
				<WatchlistAdd list={map.personalWatchlist} personal />
			{:else}
				<WatchlistAdd
					list={map.watchlist}
					disabled={shown === 'find'
						? 'Switch to a watchlist to add a system'
						: map.canWrite
							? null
							: 'Only members can add to the map watchlist'}
				/>
			{/if}
		{/snippet}
	</MapPanelHeader>
	<MapPanelContent>
		<RoutePlannerSection {map} {graph} originOnly={shown === 'find'} />
		<!-- All stay mounted, so Find keeps its picks across a look at a watchlist. -->
		<div class={shown === 'watchlist' ? '' : 'hidden'} data-testid="watchlist-shared">
			<WatchlistSection
				{map}
				list={map.watchlist}
				canEdit={map.canWrite}
				empty="Watchlist empty"
				{graph}
				{origin}
			/>
		</div>
		{#if map.signedIn}
			<div class={shown === 'personal' ? '' : 'hidden'} data-testid="watchlist-personal">
				<WatchlistSection
					{map}
					list={map.personalWatchlist}
					canEdit
					empty="Only you see this list. Add a system with +"
					{graph}
					{origin}
				/>
			</div>
		{/if}
		<div class={shown === 'find' ? '' : 'hidden'}>
			<FindSection {map} {graph} {origin} active={shown === 'find'} />
		</div>
	</MapPanelContent>
</MapPanel>
