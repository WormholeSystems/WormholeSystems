<script lang="ts">
	// Catalog types of the row's category that can spawn in this system, narrowed to the
	// linked target's class once linked. An unmatched raw scanner name shows muted.
	import type { MappedSystem } from '$lib/map/system';
	import type { Signature } from '$lib/api/types/Signature';
	import type { SignatureCatalog } from '$lib/api/types/SignatureCatalog';
	import type { SignatureTypeInfo } from '$lib/api/types/SignatureTypeInfo';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import * as Command from '$lib/components/ui/command';
	import * as Popover from '$lib/components/ui/popover';
	import { matchesTypeQuery } from '$lib/signatures/type-search';
	import { destClassMeta } from '$lib/map/classes';
	import { categoryMeta, typeById, typesForCategory } from '$lib/map/signatures';

	let {
		system,
		sig,
		catalog,
		compact,
		canWrite,
		showStaticsFirst,
		linkedClass,
		onpick,
	}: {
		system: MappedSystem;
		sig: Signature;
		catalog: SignatureCatalog;
		compact: boolean;
		canWrite: boolean;
		showStaticsFirst: boolean;
		linkedClass: number | null;
		onpick: (typeId: number | null) => void;
	} = $props();

	const isWormhole = $derived(sig.group === 'wormhole');
	const categoryId = $derived(categoryMeta(sig.group).categoryId);

	const available = $derived.by(() => {
		if (categoryId === null) return [];
		const all = typesForCategory(catalog, categoryId, system.wormhole_class_id);
		return linkedClass === null ? all : all.filter((t) => t.target_class === linkedClass);
	});

	// A static's code can also be scanned outbound, so statics come first, then K162, then
	// the rest.
	const sections = $derived.by(() => {
		if (!isWormhole) return [{ label: null, types: available }];
		const staticCodes = new Set(system.statics.map((s) => s.code));
		const statics = showStaticsFirst
			? available.filter((t) => t.signature !== null && staticCodes.has(t.signature))
			: [];
		const inStatics = new Set(statics.map((t) => t.id));
		const k162 = available.filter((t) => t.signature === 'K162' && !inStatics.has(t.id));
		const rest = available.filter((t) => t.signature !== 'K162' && !inStatics.has(t.id));
		return [
			{ label: 'Statics', types: statics },
			{ label: 'K162', types: k162 },
			{ label: 'Wormholes', types: rest },
		].filter((s) => s.types.length > 0);
	});

	const selected = $derived(typeById(catalog, sig.signature_type_id));

	let open = $state(false);
	let query = $state('');

	/** The sections, less what the query rules out; a section with nothing left goes. */
	const shown = $derived(
		sections
			.map((section) => ({
				...section,
				types: section.types.filter((t) => matchesTypeQuery(t, query)),
			}))
			.filter((section) => section.types.length > 0),
	);

	function pick(typeId: number | null) {
		onpick(typeId);
		open = false;
		query = '';
	}
</script>

{#snippet typeLabel(t: SignatureTypeInfo)}
	{#if t.signature !== null && sig.group === 'wormhole'}
		{@const dest = destClassMeta(t.target_class)}
		<span class="flex min-w-0 items-center gap-1.5 font-mono">
			{t.signature}
			<span style="color: var(--color-{dest.token})">{dest.short}</span>
			{#if t.extra}
				<span class="text-muted-foreground">({t.extra})</span>
			{/if}
		</span>
	{:else}
		<span class="truncate">{t.name}</span>
	{/if}
{/snippet}

<Popover.Root bind:open>
	<Popover.Trigger
		disabled={!canWrite || categoryId === null}
		class="flex w-full min-w-0 items-center justify-between gap-1 overflow-hidden rounded-md border
			border-input bg-transparent px-2 text-xs shadow-xs disabled:cursor-not-allowed
			disabled:opacity-50 {compact ? 'h-5 py-0' : 'h-6'}"
		data-testid="sig-type"
	>
		{#if selected}
			{@render typeLabel(selected)}
		{:else if sig.name}
			<span class="truncate text-muted-foreground">{sig.name}</span>
		{:else}
			<span class="text-muted-foreground">Type</span>
		{/if}
		<ChevronDownIcon class="size-3 shrink-0 opacity-50" />
	</Popover.Trigger>
	<Popover.Content class="w-64 p-0" align="start">
		<!-- The list filters itself, so the Command primitive must not filter it again on
		     its own rules: `shouldFilter={false}`. Ours matches the destination class as
		     well as the code, which is how people ask for these holes. -->
		<Command.Root shouldFilter={false}>
			<!-- Not "Code, ...": password managers classify a field by keywords in its
			     placeholder, and "code" reads as a verification code, so Apple's Passwords
			     offers to fill the box. -->
			<Command.Input
				placeholder="K162, name or where it leads…"
				bind:value={query}
				class="h-8 text-xs"
				data-testid="sig-type-search"
			/>
			<Command.List class="max-h-64">
				<!-- Our own, because `shouldFilter={false}` means the primitive does not know
				     what the query ruled out and so never reaches its own empty state. -->
				{#if shown.length === 0}
					<div class="py-4 text-center text-xs text-muted-foreground">No type matches</div>
				{/if}
				<Command.Group>
					<Command.Item value="none" onSelect={() => pick(null)} class="text-xs">
						<span class="text-muted-foreground">Unknown</span>
					</Command.Item>
				</Command.Group>
				{#each shown as section, i (section.label ?? i)}
					<Command.Group heading={section.label ?? undefined}>
						{#each section.types as t (t.id)}
							<Command.Item value={String(t.id)} onSelect={() => pick(t.id)} class="text-xs">
								{@render typeLabel(t)}
							</Command.Item>
						{/each}
					</Command.Group>
				{/each}
			</Command.List>
		</Command.Root>
	</Popover.Content>
</Popover.Root>
