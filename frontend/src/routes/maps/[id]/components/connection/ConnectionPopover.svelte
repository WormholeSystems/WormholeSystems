<script lang="ts">
	// Connection details, anchored at the click on the edge. Read-mostly: the only writes are
	// in the jump log, every other mutation stays in the context menu.
	import type { Signature } from '$lib/api/types/Signature';
	import { createQuery } from '@tanstack/svelte-query';

	import { q } from '$lib/api/queries';
	import * as Popover from '$lib/components/ui/popover';
	import { typeById } from '$lib/map/signatures';
	import ConnectionStatus from './ConnectionStatus.svelte';
	import MassTracking from './MassTracking.svelte';
	import SignatureSection from './SignatureSection.svelte';
	import WormholeProperties from './WormholeProperties.svelte';
	import type { MapState } from '../../state/map-state.svelte';

	let { map }: { map: MapState } = $props();

	const catalogQuery = createQuery(() => q.signatureCatalog());
	const catalog = $derived(catalogQuery.data ?? null);

	const popover = $derived(map.connectionPopover);
	// Resolved live so refetches keep it current; a deleted connection closes it.
	const connection = $derived(
		popover === null ? null : (map.connections.all.find((c) => c.id === popover.id) ?? null),
	);
	$effect(() => {
		if (popover !== null && connection === null) map.connectionPopover = null;
	});

	const source = $derived(
		connection === null
			? null
			: (map.systems.all.find((s) => s.id === connection.from_system) ?? null),
	);
	const target = $derived(
		connection === null
			? null
			: (map.systems.all.find((s) => s.id === connection.to_system) ?? null),
	);
	const sigs = $derived(
		connection === null ? [] : map.signatures.all.filter((s) => s.connection_id === connection.id),
	);

	function codeOf(sig: Signature): string | null {
		return (catalog && typeById(catalog, sig.signature_type_id)?.signature) ?? null;
	}
	// An untyped signature counts as the outbound side.
	const outSig = $derived(sigs.find((s) => !codeOf(s)?.startsWith('K162')) ?? null);
	const inSig = $derived(sigs.find((s) => codeOf(s)?.startsWith('K162')) ?? null);

	// Physics prefer the outbound side's type; the K162 side is a coarse fallback.
	const physics = $derived.by(() => {
		if (!catalog) return null;
		const out = outSig === null ? null : typeById(catalog, outSig.signature_type_id);
		if (out?.total_mass != null) return out;
		const k162 = inSig === null ? null : typeById(catalog, inSig.signature_type_id);
		if (k162?.total_mass != null) return k162;
		return out ?? k162;
	});

	const canWrite = $derived(map.canWrite);

	const open = $derived(popover !== null && connection !== null);
</script>

{#if popover !== null && connection !== null && source !== null && target !== null && catalog !== null}
	{#key connection.id}
		<Popover.Root
			{open}
			onOpenChange={(o) => {
				if (!o) map.connectionPopover = null;
			}}
		>
			<Popover.Trigger
				class="pointer-events-none fixed size-0"
				style="left: {popover.x}px; top: {popover.y}px"
				tabindex={-1}
			/>
			<Popover.Content
				class="max-h-[85vh] w-60 overflow-y-auto"
				data-testid="connection-popover"
				onpointerdown={(ev: PointerEvent) => ev.stopPropagation()}
				oncontextmenu={(ev: MouseEvent) => ev.stopPropagation()}
				onOpenAutoFocus={(ev) => ev.preventDefault()}
			>
				<!-- ignoreNonKeyboardFocus: without it the popover's auto-focus lands on the
				     first tooltip trigger, popping a tooltip whose dismiss layer then eats
				     the first outside click. -->
				<div class="space-y-3">
					{#if outSig !== null}
						<SignatureSection title="Out Sig" sig={outSig} {catalog} />
					{/if}
					{#if inSig !== null}
						<SignatureSection title="In Sig" sig={inSig} {catalog} />
					{/if}
					{#if outSig === null && inSig === null}
						<div class="space-y-1">
							<div class="py-2 text-center text-xs text-muted-foreground">
								No signatures assigned
							</div>
						</div>
					{/if}
					<ConnectionStatus {connection} {sigs} />
					{#if physics !== null}
						<WormholeProperties type={physics} />
					{/if}
					{#if connection.kind === 'wormhole'}
						<MassTracking {map} {connection} {source} {target} {physics} {canWrite} />
					{/if}
				</div>
			</Popover.Content>
		</Popover.Root>
	{/key}
{/if}
