<script lang="ts">
	// Catalog types of the row's category that can spawn in this system, narrowed to the
	// linked target's class once linked. An unmatched raw scanner name shows muted.
	import type { MappedSystem } from '$lib/map/system';
	import type { Signature } from '$lib/api/types/Signature';
	import type { SignatureCatalog } from '$lib/api/types/SignatureCatalog';
	import type { SignatureTypeInfo } from '$lib/api/types/SignatureTypeInfo';
	import * as Select from '$lib/components/ui/select';
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

	function pick(value: string) {
		if (value === 'none') onpick(null);
		else onpick(Number(value));
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

<Select.Root
	type="single"
	value={sig.signature_type_id === null ? '' : String(sig.signature_type_id)}
	onValueChange={pick}
	disabled={!canWrite || categoryId === null}
>
	<Select.Trigger
		class="w-full min-w-0 overflow-hidden text-xs {compact ? '!h-5 !py-0' : ''}"
		data-testid="sig-type"
	>
		{#if selected}
			{@render typeLabel(selected)}
		{:else if sig.name}
			<span class="truncate text-muted-foreground">{sig.name}</span>
		{:else}
			<span class="text-muted-foreground">Type</span>
		{/if}
	</Select.Trigger>
	<Select.Content class="max-h-72">
		<Select.Group>
			<Select.Item value="none" class="text-xs">
				<span class="text-muted-foreground">Unknown</span>
			</Select.Item>
		</Select.Group>
		{#each sections as section, i (section.label ?? i)}
			<Select.Group>
				{#if section.label}
					<Select.GroupHeading class="text-xs text-muted-foreground">
						{section.label}
					</Select.GroupHeading>
				{/if}
				{#each section.types as t (t.id)}
					<Select.Item value={String(t.id)} class="text-xs" label={t.name}>
						{@render typeLabel(t)}
					</Select.Item>
				{/each}
			</Select.Group>
		{/each}
	</Select.Content>
</Select.Root>
