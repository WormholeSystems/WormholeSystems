<script lang="ts">
	// The signature panel's own rows, driven by static data and given no actions, so what a
	// visitor sees is what the product renders rather than a mock-up of it.
	import { createQuery } from '@tanstack/svelte-query';

	import { q } from '$lib/api/queries';
	import { tickingMs } from '$lib/now.svelte';
	import MapPanelHeader from '$lib/components/map-panel/MapPanelHeader.svelte';
	import SignatureColumns from '$lib/components/SignatureColumns.svelte';
	import type { SignatureContext } from '$lib/map/signature-context';
	import SignatureRow from '$lib/components/SignatureRow.svelte';
	import { DEMO_CONNECTIONS, DEMO_SIGNATURES, DEMO_SYSTEMS, HOME_SYSTEM } from './demo-chain';

	// No `actions`, so the rows cannot write anything even if something were clicked.
	const ctx: SignatureContext = {
		naming: null,
		systems: DEMO_SYSTEMS,
		connections: DEMO_CONNECTIONS,
		systemById: new Map(DEMO_SYSTEMS.map((s) => [s.id, s])),
		connectionById: new Map(DEMO_CONNECTIONS.map((c) => [c.id, c])),
		sigs: DEMO_SIGNATURES,
	};
	const system = HOME_SYSTEM;

	const catalogQuery = createQuery(() => q.signatureCatalog());
	const catalog = $derived(catalogQuery.data ?? null);
	const clock = tickingMs(30_000);
</script>

<div class="overflow-hidden rounded border border-border bg-card">
	<MapPanelHeader>
		Signatures
		<span class="ml-1 text-amber-400">{DEMO_SIGNATURES.length}</span>
	</MapPanelHeader>
	<SignatureColumns />
	{#if catalog}
		{#each DEMO_SIGNATURES as sig (sig.id)}
			<SignatureRow
				{ctx}
				{system}
				{sig}
				{catalog}
				compact={false}
				canWrite={false}
				showStaticsFirst={false}
				status={sig.signature_id === 'GHI-789' ? 'new' : null}
				now={clock.current}
			/>
		{/each}
	{:else}
		<!-- Holds the row heights while the catalog arrives, so the band does not jump. -->
		{#each DEMO_SIGNATURES as sig (sig.id)}
			<div class="h-9 border-b border-border/30"></div>
		{/each}
	{/if}
</div>
