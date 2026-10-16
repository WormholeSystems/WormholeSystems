<script lang="ts">
	// Links a signature to one of the system's connections. Unclaimed ones list first, and a
	// typed signature filters to connections whose far end matches the type's destination.
	import type { MapConnection } from '$lib/api/types/MapConnection';
	import type { MapSystemView } from '$lib/api/types/MapSystemView';
	import { systemName, type MappedSystem } from '$lib/map/system';
	import type { Signature } from '$lib/api/types/Signature';
	import type { SignatureCatalog } from '$lib/api/types/SignatureCatalog';
	import * as Select from '$lib/components/ui/select';
	import ClassBadge from '$lib/components/ClassBadge.svelte';
	import { typeById } from '$lib/map/signatures';
	import type { SignatureContext } from '$lib/map/signature-context';

	let {
		ctx,
		system,
		sig,
		catalog,
		compact,
		canWrite,
	}: {
		ctx: SignatureContext;
		system: MappedSystem;
		sig: Signature;
		catalog: SignatureCatalog;
		compact: boolean;
		canWrite: boolean;
	} = $props();

	function target(c: MapConnection): MapSystemView | null {
		const otherPid = c.from_system === system.id ? c.to_system : c.from_system;
		return ctx.systemById.get(otherPid) ?? null;
	}

	const candidates = $derived.by(() => {
		const type = typeById(catalog, sig.signature_type_id);
		const targetClass = type?.target_class ?? null;
		return ctx.connections
			.filter((c) => c.from_system === system.id || c.to_system === system.id)
			.map((c) => ({ conn: c, target: target(c) }))
			.filter((e) => e.target !== null)
			.filter(
				(e) =>
					targetClass === null ||
					(e.target?.kind === 'system' && e.target.wormhole_class_id === targetClass),
			)
			.sort((a, b) => {
				const aa = a.target?.alias ?? null;
				const bb = b.target?.alias ?? null;
				if (aa !== null && bb !== null) return aa.localeCompare(bb);
				if (aa !== null) return -1;
				if (bb !== null) return 1;
				const an = a.target ? (systemName(a.target) ?? '') : '';
				return an.localeCompare(b.target ? (systemName(b.target) ?? '') : '');
			});
	});

	// Connections another signature in this system already claims.
	const claimed = $derived(
		new Set(
			ctx.sigs
				.filter(
					(s) =>
						s.solar_system_id === system.solar_system_id &&
						s.id !== sig.id &&
						s.connection_id !== null,
				)
				.map((s) => s.connection_id),
		),
	);

	const unclaimed = $derived(candidates.filter((e) => !claimed.has(e.conn.id)));
	const connected = $derived(candidates.filter((e) => claimed.has(e.conn.id)));
	const selected = $derived(
		sig.connection_id === null
			? null
			: (candidates.find((e) => e.conn.id === sig.connection_id) ?? null),
	);

	function pick(value: string) {
		if (value === 'unlink') {
			if (sig.connection_id !== null) ctx.actions?.unlink(sig.id);
			return;
		}
		const connectionId = Number(value);
		if (!Number.isFinite(connectionId) || connectionId === sig.connection_id) return;
		ctx.actions?.link(sig.id, connectionId);
	}
</script>

{#snippet connLabel(entry: { conn: MapConnection; target: MapSystemView | null })}
	{@const t = entry.target}
	{#if t}
		{@const mapped = t.kind === 'system' ? t : null}
		<span class="inline-flex min-w-0 items-center gap-1">
			<ClassBadge
				classId={mapped?.wormhole_class_id ?? null}
				security={mapped?.security_status ?? null}
				class="w-5 shrink-0 text-center"
			/>
			{#if t.alias}
				<span class="shrink-0 font-medium">{t.alias}</span>
			{/if}
			<span class="truncate {t.alias ? 'text-muted-foreground' : ''}">
				{mapped?.name ?? 'Unmapped'}
			</span>
			<span class="shrink-0 text-muted-foreground/60">{mapped?.region ?? ''}</span>
		</span>
	{/if}
{/snippet}

<Select.Root
	type="single"
	value={sig.connection_id === null ? '' : String(sig.connection_id)}
	onValueChange={pick}
	disabled={!canWrite}
>
	<Select.Trigger
		class="w-full min-w-0 overflow-hidden text-xs {compact ? '!h-5 !py-0' : ''}"
		data-testid="sig-connection"
	>
		{#if selected}
			{@render connLabel(selected)}
		{:else}
			<span class="truncate text-muted-foreground">Connection</span>
		{/if}
	</Select.Trigger>
	<Select.Content class="max-h-72">
		<Select.Group>
			<Select.Item value="unlink" class="text-xs">
				<span class="text-muted-foreground">Unknown</span>
			</Select.Item>
		</Select.Group>
		{#if unclaimed.length > 0}
			<Select.Separator />
			<Select.Group>
				<Select.GroupHeading class="text-xs text-muted-foreground">Connections</Select.GroupHeading>
				{#each unclaimed as entry (entry.conn.id)}
					<Select.Item
						value={String(entry.conn.id)}
						class="text-xs"
						label={(entry.target && systemName(entry.target)) ?? String(entry.conn.id)}
					>
						{@render connLabel(entry)}
					</Select.Item>
				{/each}
			</Select.Group>
		{/if}
		{#if connected.length > 0}
			<Select.Separator />
			<Select.Group>
				<Select.GroupHeading class="text-xs text-muted-foreground">Connected</Select.GroupHeading>
				{#each connected as entry (entry.conn.id)}
					<Select.Item
						value={String(entry.conn.id)}
						class="text-xs"
						label={(entry.target && systemName(entry.target)) ?? String(entry.conn.id)}
					>
						{@render connLabel(entry)}
					</Select.Item>
				{/each}
			</Select.Group>
		{/if}
	</Select.Content>
</Select.Root>
