<script lang="ts">
	import { page } from '$app/state';

	import MapScreen from './components/MapScreen.svelte';
	import type { MapSeed } from './state/map-queries.svelte';

	const mapId = $derived(Number(page.params.id) || 0);
	let { data }: { data: MapSeed } = $props();
</script>

<!-- Keyed so each map gets its own MapScreen instance: switching maps rebuilds the whole
     object graph, and constructors run at component init where queries are legal. -->
{#key mapId}
	<MapScreen {mapId} seed={data} signedIn={page.data.me != null} />
{/key}
