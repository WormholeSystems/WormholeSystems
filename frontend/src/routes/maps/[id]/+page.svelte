<script lang="ts">
	import type { Component } from 'svelte';

	import { page } from '$app/state';

	import type { MapSeed } from './state/map-queries.svelte';

	const mapId = $derived(Number(page.params.id) || 0);
	let { data }: { data: MapSeed } = $props();

	// The load runs on the server, so a map that is gone or not yours answers with its real
	// status and error page. The screen itself is only ever drawn in the browser, and only
	// imported there: the per-user panel arrangement arrives from a separate request, so a
	// server render would paint the built-in layout and then shove every tile into place,
	// and parts of the tree (the notes' sanitiser among them) need a window to even load.
	type Props = { mapId: number; seed: MapSeed; signedIn: boolean };
	let Screen = $state<Component<Props> | null>(null);
	$effect(() => {
		void import('./components/MapScreen.svelte').then((m) => (Screen = m.default));
	});
</script>

<!-- Keyed so each map gets its own MapScreen instance: switching maps rebuilds the whole
     object graph, and constructors run at component init where queries are legal. -->
{#if Screen}
	{#key mapId}
		<Screen {mapId} seed={data} signedIn={page.data.me != null} />
	{/key}
{/if}
