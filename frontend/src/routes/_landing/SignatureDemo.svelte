<script lang="ts">
	// The signature panel's own rows and its own paste diff, on static data. Pasting runs
	// the same parser and the same new/refreshed/gone classification the map does, so a
	// visitor can paste a scan of their own and see exactly what the panel would make of it.
	// Nothing is written anywhere: the rows get no actions.
	import ClipboardPasteIcon from '@lucide/svelte/icons/clipboard-paste';
	import RotateCcwIcon from '@lucide/svelte/icons/rotate-ccw';
	import { createQuery } from '@tanstack/svelte-query';

	import { q } from '$lib/api/queries';
	import type { PastedSignature } from '$lib/api/types/PastedSignature';
	import type { Signature } from '$lib/api/types/Signature';
	import type { SignatureCatalog } from '$lib/api/types/SignatureCatalog';
	import MapPanelHeader from '$lib/components/map-panel/MapPanelHeader.svelte';
	import SignatureColumns from '$lib/components/SignatureColumns.svelte';
	import SignatureRow from '$lib/components/SignatureRow.svelte';
	import { Button } from '$lib/components/ui/button';
	import { loadScanNames, parseScan } from '$lib/map/signatures';
	import type { SignatureContext } from '$lib/map/signature-context';
	import { tickingMs } from '$lib/now.svelte';
	import { pasteStatus } from '$lib/signatures/paste-diff';
	import Frame from './Frame.svelte';
	import {
		DEMO_CONNECTIONS,
		DEMO_SCAN,
		DEMO_SYSTEMS,
		HOME_SYSTEM,
		DEMO_WORMHOLE_TYPES,
		demoSignature,
		demoSignatures,
	} from './demo-chain';

	const catalogQuery = createQuery(() => q.signatureCatalog());
	const catalog = $derived(catalogQuery.data ?? null);
	const clock = tickingMs(30_000);

	/** The starting rows, with their wormhole types resolved against the loaded catalog. */
	function initial(current: SignatureCatalog | null): Signature[] {
		return demoSignatures().map((s) => {
			const type = DEMO_WORMHOLE_TYPES[s.signature_id];
			const id = type && current?.types.find((t) => t.name === type)?.id;
			return id ? { ...s, signature_type_id: id } : s;
		});
	}

	let sigs = $state<Signature[]>(initial(null));
	// The rows only render once the catalog is here, so typing them then changes nothing
	// anybody has seen.
	let typed = false;
	$effect(() => {
		if (catalog && !typed && pastedIds === null) {
			typed = true;
			sigs = initial(catalog);
		}
	});
	let preIds = $state<Set<string>>(new Set());
	let pastedIds = $state<Set<string> | null>(null);
	let error = $state<string | null>(null);

	const rows = $derived(sigs.toSorted((a, b) => a.signature_id.localeCompare(b.signature_id)));
	const ctx: SignatureContext = $derived({
		naming: null,
		systems: DEMO_SYSTEMS,
		connections: DEMO_CONNECTIONS,
		systemById: new Map(DEMO_SYSTEMS.map((s) => [s.id, s])),
		connectionById: new Map(DEMO_CONNECTIONS.map((c) => [c.id, c])),
		sigs,
	});

	const counts = $derived.by(() => {
		const c = { new: 0, updated: 0, deleted: 0 };
		for (const s of sigs) {
			const status = pasteStatus(s, preIds, pastedIds);
			if (status) c[status]++;
		}
		return c;
	});

	/**
	 * What the server makes of a pasted row (src/maps/signatures/paste.rs), minus the round
	 * trip: the group is the scanner's if it has one, a wormhole keeps its type and name, and
	 * a site takes whatever the scan says it is now.
	 */
	function merge(had: Signature, p: PastedSignature, now: string): Signature {
		const group = p.group ?? had.group;
		const wormhole = group === 'wormhole';
		const typeId = wormhole
			? had.signature_type_id
			: (p.signature_type_id ?? (p.name !== undefined ? null : had.signature_type_id));
		return {
			...had,
			group,
			signature_type_id: typeId,
			name: wormhole ? (had.name ?? p.name ?? null) : (p.name ?? had.name),
			connection_id: wormhole ? had.connection_id : null,
			updated_at: now,
		};
	}

	/** What the panel does with a paste: refresh what it knew, add the rest, and mark both. */
	function apply(pasted: PastedSignature[]) {
		const now = new Date().toISOString();
		const before = new Map(sigs.map((s) => [s.signature_id, s]));
		const next = [...sigs];
		for (const p of pasted) {
			const had = before.get(p.signature_id);
			if (had) next[next.indexOf(had)] = merge(had, p, now);
			else
				next.push(
					demoSignature({
						signature_id: p.signature_id,
						group: p.group ?? 'unknown',
						signature_type_id: p.signature_type_id ?? null,
						name: p.name ?? null,
					}),
				);
		}
		preIds = new Set(before.keys());
		pastedIds = new Set(pasted.map((p) => p.signature_id));
		sigs = next;
	}

	async function paste(text: string, current: SignatureCatalog) {
		const parsed = parseScan(text, current, await loadScanNames());
		if (parsed.length === 0) {
			error = 'That did not look like a probe scan. Copy the rows from the scanner window.';
			return;
		}
		error = null;
		apply(parsed);
	}

	function reset() {
		sigs = initial(catalog);
		preIds = new Set();
		pastedIds = null;
		error = null;
	}

	function onpaste(event: ClipboardEvent) {
		const text = event.clipboardData?.getData('text/plain');
		if (!text || !catalog) return;
		event.preventDefault();
		void paste(text, catalog);
	}
</script>

<Frame>
	<!-- Focusable so a pasted scan lands here; the rows themselves cannot be edited. -->
	<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
	<div
		class="bg-card outline-none focus-visible:ring-2 focus-visible:ring-ring/40 focus-visible:ring-inset"
		tabindex="0"
		role="region"
		aria-label="Signature panel demo. Focus it and paste a probe scan."
		data-testid="signature-demo"
		{onpaste}
	>
		<MapPanelHeader>
			Signatures
			<span class="ml-1 text-amber-400">{sigs.length}</span>
			<span class="ml-2 normal-case">Turnur</span>
			{#snippet actions()}
				{#if pastedIds}
					<Button size="xs" variant="ghost" onclick={reset}>
						<RotateCcwIcon data-icon="inline-start" />
						Reset
					</Button>
				{:else}
					<Button
						size="xs"
						variant="outline"
						disabled={!catalog}
						onclick={() => catalog && paste(DEMO_SCAN, catalog)}
						data-testid="signature-demo-paste"
					>
						<ClipboardPasteIcon data-icon="inline-start" />
						Paste a new scan
					</Button>
				{/if}
			{/snippet}
		</MapPanelHeader>
		<!-- The rows are the desktop panel's: on a phone they keep that width and scroll
		     sideways, rather than squeezing the type and connection down to bare chevrons. -->
		<div class="overflow-x-auto">
			<div class="min-w-[640px]">
				<SignatureColumns />
				<!-- Room for the rows the demo scan adds, so pasting it does not move the page. -->
				<div class="min-h-[18rem]">
					{#if catalog}
						{#each rows as sig (sig.id)}
							<SignatureRow
								{ctx}
								system={HOME_SYSTEM}
								{sig}
								{catalog}
								compact={false}
								canWrite={false}
								showStaticsFirst={false}
								status={pasteStatus(sig, preIds, pastedIds)}
								now={clock.current}
							/>
						{/each}
					{/if}
				</div>
			</div>
		</div>
		<p
			class="flex min-h-10 items-center border-t border-border/50 px-3 font-mono text-[10px] tracking-[0.15em] text-muted-foreground uppercase"
			aria-live="polite"
		>
			{#if error}
				<span class="tracking-normal normal-case text-destructive">{error}</span>
			{:else if pastedIds}
				<span class="text-green-500">{counts.new} new</span>
				<span class="mx-1.5 text-muted-foreground/50">·</span>
				<span class="text-amber-500">{counts.updated} seen again</span>
				<span class="mx-1.5 text-muted-foreground/50">·</span>
				<span class="text-red-500">{counts.deleted} gone</span>
			{:else}
				Or click here and paste a scan of your own
			{/if}
		</p>
	</div>
</Frame>
