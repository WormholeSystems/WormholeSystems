<script lang="ts">
	// Route planner, shared watchlist, and closest-systems Find, in that order. One origin
	// drives the watchlist and Find distances: the route From, else the active system, else
	// the tracked character's location.
	import MapPanel from '$lib/components/map-panel/MapPanel.svelte';
	import MapPanelContent from '$lib/components/map-panel/MapPanelContent.svelte';
	import MapPanelHeader from '$lib/components/map-panel/MapPanelHeader.svelte';
	import type { MapState } from '../../state/map-state.svelte';
	import FindSection from './navigation/FindSection.svelte';
	import RoutePlannerSection from './navigation/RoutePlannerSection.svelte';
	import RouteSettings from './navigation/RouteSettings.svelte';
	import WatchlistAdd from './navigation/WatchlistAdd.svelte';
	import WatchlistSection from './navigation/WatchlistSection.svelte';

	let { map }: { map: MapState } = $props();

	const PREF_LABELS = {
		shorter: 'Shortest',
		safer: 'Safer',
		less_secure: 'Less Secure',
	} satisfies Record<string, string>;

	const graph = $derived(map.route.graph);
	const origin = $derived(map.routeOrigin);
</script>

<MapPanel testid="navigation-card">
	<MapPanelHeader>
		Navigation
		<span class="ml-1 text-muted-foreground/60 normal-case">
			{PREF_LABELS[map.userSettings?.route_preference ?? 'shorter']}
		</span>
		{#snippet actions()}
			<RouteSettings {map} />
			{#if map.canWrite}
				<WatchlistAdd {map} />
			{/if}
		{/snippet}
	</MapPanelHeader>
	<MapPanelContent>
		<RoutePlannerSection {map} {graph} />
		<WatchlistSection {map} {graph} {origin} />
		<FindSection {map} {graph} {origin} />
	</MapPanelContent>
</MapPanel>
