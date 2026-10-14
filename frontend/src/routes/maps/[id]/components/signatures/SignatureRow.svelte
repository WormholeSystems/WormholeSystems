<script lang="ts">
	// One signature row. Paste-diff status tints ride on data attributes.
	import CopyIcon from '@lucide/svelte/icons/copy';
	import MoreVerticalIcon from '@lucide/svelte/icons/more-vertical';
	import TrashIcon from '@lucide/svelte/icons/trash-2';

	import { toast } from 'svelte-sonner';
	import { copyText } from '$lib/clipboard';

	import { aliasTargetKind, suggestAlias } from '$lib/naming/alias';
	import { formatsFromNaming, signatureBookmark } from '$lib/naming/bookmark';
	import { classMeta, isWormholeClass } from '$lib/map/classes';
	import type { MappedSystem } from '$lib/map/system';
	import type { MassStatus } from '$lib/api/types/MassStatus';
	import type { Signature } from '$lib/api/types/Signature';
	import type { TimeStatus } from '$lib/api/types/TimeStatus';
	import type { SignatureCatalog } from '$lib/api/types/SignatureCatalog';
	import type { SignatureGroup } from '$lib/api/types/SignatureGroup';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import * as Select from '$lib/components/ui/select';
	import { LIFETIME_OPTIONS, SIGNATURE_MASS_OPTIONS } from '$lib/map/connection-status';
	import { CATEGORIES, categoryMeta, typeById } from '$lib/map/signatures';
	import type { SignatureContext, SignaturePatch } from '$lib/map/signature-context';
	import ConnectionInput from './ConnectionInput.svelte';
	import TimeDetails from './TimeDetails.svelte';
	import TypeInput from './TypeInput.svelte';

	let {
		ctx,
		system,
		sig,
		catalog,
		compact,
		canWrite,
		showStaticsFirst,
		status,
		now,
	}: {
		ctx: SignatureContext;
		system: MappedSystem;
		sig: Signature;
		catalog: SignatureCatalog;
		compact: boolean;
		canWrite: boolean;
		showStaticsFirst: boolean;
		status: 'new' | 'updated' | 'deleted' | null;
		/** Epoch ms from the panel's shared clock. */
		now: number;
	} = $props();

	const isWormhole = $derived(sig.group === 'wormhole');
	const cat = $derived(categoryMeta(sig.group));

	const connection = $derived(
		sig.connection_id === null
			? null
			: (ctx.connections.find((c) => c.id === sig.connection_id) ?? null),
	);
	// The linked connection's far end, for type narrowing and the bookmark class.
	const linkedTarget = $derived.by(() => {
		if (!connection) return null;
		const otherPid =
			connection.from_system === system.id ? connection.to_system : connection.from_system;
		return ctx.systems.find((s) => s.id === otherPid) ?? null;
	});
	/** The far end once it is a system: a hole nobody has been through has none of this. */
	const linkedSystem = $derived(linkedTarget?.kind === 'system' ? linkedTarget : null);

	// Inline ID editing: alphanumerics only, uppercased, dash after 3 characters.
	let editingId = $state(false);
	let idDraft = $state('');
	let idInput = $state<HTMLInputElement | null>(null);

	function startEditId() {
		if (!canWrite) return;
		idDraft = sig.signature_id;
		editingId = true;
		setTimeout(() => {
			idInput?.focus();
			idInput?.select();
		});
	}

	function formatId(raw: string): string {
		const clean = raw.replace(/[^a-zA-Z0-9]/g, '').toUpperCase();
		return clean.length >= 4 ? `${clean.slice(0, 3)}-${clean.slice(3, 6)}` : clean;
	}

	function saveId() {
		const value = idDraft.trim();
		editingId = false;
		if (value.length === 7 && value !== sig.signature_id) {
			update({ signature_id: value });
		}
	}

	function update(patch: SignaturePatch) {
		ctx.actions?.update(sig.id, patch);
	}

	function pickCategory(value: string) {
		const group = value as SignatureGroup;
		if (group === sig.group) return;
		update({ group });
	}

	// The bookmark names the far end of the hole ("where this leads"), not where you are
	// standing. Until the hole is mapped, the signature type's promised class is all we know.
	function copyBookmark() {
		const type = typeById(catalog, sig.signature_type_id);
		const far = linkedSystem;
		const farClass = far?.wormhole_class_id ?? type?.target_class ?? null;
		const text = signatureBookmark(
			sig,
			type,
			far,
			linkedTarget?.alias ?? guessAlias(farClass, far?.security_status ?? null),
			system.alias,
			formatsFromNaming(ctx.naming),
		);
		void copyText(text, { silent: true });
		toast.success('Bookmark copied', { description: text });
	}

	// A scanner bookmarks every hole before opening any, so each copy has to take the next
	// name in the chain rather than all of them claiming the first. Written onto the ghost
	// when there is one, so the next row sees it taken; a hole with no placement at all
	// gets the name on the clipboard only.
	function guessAlias(farClass: number | null, farSecurity: number | null): string | null {
		const targetIsWormhole = isWormholeClass(farClass);
		const alias = suggestAlias({
			parentAlias: system.alias,
			targetIsWormhole,
			originIsWormhole: isWormholeClass(system.wormhole_class_id),
			aliases: ctx.systems.map((s) => s.alias).filter((alias): alias is string => Boolean(alias)),
			scheme: ctx.naming?.alias_scheme,
			targetKind: aliasTargetKind(targetIsWormhole, classMeta(farClass, farSecurity).short),
			ignoredAlias: ctx.naming?.ignored_alias,
		});
		if (alias && linkedTarget && canWrite) ctx.actions?.setAlias(linkedTarget, alias);
		return alias;
	}

	function remove() {
		ctx.actions?.remove(sig.id);
	}

	function togglePreserveMass() {
		if (!connection) return;
		ctx.actions?.setPreserveMass(connection.id, !connection.preserve_mass);
	}

	/** The widget's value as a mass status. Its own "fresh" choice means no status at all. */
	function massFrom(value: string): MassStatus | null {
		return value === 'reduced' || value === 'critical' ? value : null;
	}

	/** The same for lifetime, where "healthy" means no status. */
	function timeFrom(value: string): TimeStatus | null {
		return value === 'eol' || value === 'critical' || value === 'expired' ? value : null;
	}
</script>

<div
	class="flex items-center gap-2 border-b border-border/30 px-3 hover:bg-muted/30 data-[status=deleted]:bg-red-500/10 data-[status=new]:bg-green-500/10 data-[status=updated]:bg-amber-500/15 {compact
		? 'py-0.5'
		: 'py-1.5'}"
	data-testid="sig-row"
	data-sig={sig.signature_id}
	data-status={status}
>
	<div class="w-16 shrink-0">
		{#if editingId}
			<Input
				bind:ref={idInput}
				value={idDraft}
				oninput={(e) => (idDraft = formatId(e.currentTarget.value))}
				onblur={saveId}
				onkeydown={(e) => {
					if (e.key === 'Enter') saveId();
					if (e.key === 'Escape') editingId = false;
				}}
				class="w-full rounded-none border-border/50 bg-background/50 px-1.5 font-mono text-xs uppercase {compact
					? 'h-5'
					: 'h-6'}"
				maxlength={7}
				placeholder="XXX-XXX"
			/>
		{:else}
			<button
				class="flex items-center font-mono text-xs hover:text-amber-400 {canWrite
					? 'cursor-pointer'
					: 'cursor-default'} {compact ? 'h-5' : 'h-6'}"
				onclick={startEditId}
			>
				{sig.signature_id || '---'}
			</button>
		{/if}
	</div>

	<div class="w-20 shrink-0">
		<Select.Root
			type="single"
			value={sig.group === 'unknown' ? '' : sig.group}
			onValueChange={pickCategory}
			disabled={!canWrite}
		>
			<Select.Trigger
				class="w-full min-w-0 overflow-hidden text-xs {compact ? '!h-5 !py-0' : ''}"
				data-testid="sig-category"
			>
				{#if sig.group === 'unknown'}
					<span class="text-muted-foreground">Category</span>
				{:else}
					<span class="flex min-w-0 items-center gap-1">
						<cat.icon class="size-3 shrink-0 {cat.color}" />
						<span class="truncate">{cat.abbrev}</span>
					</span>
				{/if}
			</Select.Trigger>
			<Select.Content>
				<Select.Group>
					{#each CATEGORIES.filter((c) => c.categoryId !== null) as c (c.group)}
						<Select.Item value={c.group} class="text-xs" label={c.label}>
							<span class="flex items-center gap-1.5">
								<c.icon class="size-3 shrink-0 {c.color}" />
								{c.label}
							</span>
						</Select.Item>
					{/each}
				</Select.Group>
			</Select.Content>
		</Select.Root>
	</div>

	<div class="min-w-0 flex-1 {isWormhole ? 'max-w-44' : ''}">
		<TypeInput
			{system}
			{sig}
			{catalog}
			{compact}
			{canWrite}
			{showStaticsFirst}
			linkedClass={linkedSystem?.wormhole_class_id ?? null}
			onpick={(typeId) => update({ signature_type_id: typeId })}
		/>
	</div>

	<!-- Wormhole rows only; on a site the type cell absorbs the column. -->
	{#if isWormhole}
		<div class="min-w-0 flex-1">
			<ConnectionInput {ctx} {system} {sig} {catalog} {compact} {canWrite} />
		</div>
	{/if}

	<div class="w-10 shrink-0 text-right">
		<TimeDetails {sig} {connection} {compact} {now} />
	</div>

	<div class="flex w-12 shrink-0 items-center justify-end gap-1">
		{#if isWormhole}
			<Button
				variant="ghost"
				size="icon"
				class="size-6 text-muted-foreground hover:text-foreground"
				aria-label="Copy bookmark"
				onclick={copyBookmark}
			>
				<CopyIcon class="size-3.5" />
			</Button>
		{/if}
		{#if canWrite}
			<DropdownMenu.Root>
				<DropdownMenu.Trigger>
					{#snippet child({ props })}
						<Button
							{...props}
							variant="ghost"
							size="icon"
							class="text-muted-foreground hover:text-foreground {compact ? 'size-5' : 'size-6'}"
							aria-label="Signature menu"
						>
							<MoreVerticalIcon class="size-3.5" />
						</Button>
					{/snippet}
				</DropdownMenu.Trigger>
				<DropdownMenu.Content align="end" class="w-44">
					{#if isWormhole}
						<DropdownMenu.RadioGroup
							value={sig.mass_status ?? 'unknown'}
							onValueChange={(v) => update({ mass_status: massFrom(v) })}
						>
							{#each SIGNATURE_MASS_OPTIONS as opt (opt.value)}
								<DropdownMenu.RadioItem value={opt.value} class="text-xs">
									<span class="flex items-center gap-2">
										<span class="inline-block size-2 rounded-full {opt.dot}"></span>
										{opt.label}
									</span>
								</DropdownMenu.RadioItem>
							{/each}
						</DropdownMenu.RadioGroup>
						<DropdownMenu.Separator />
						<DropdownMenu.RadioGroup
							value={sig.time_status ?? 'stable'}
							onValueChange={(v) => update({ time_status: timeFrom(v) })}
						>
							{#each LIFETIME_OPTIONS as opt (opt.value)}
								<DropdownMenu.RadioItem value={opt.value} class="text-xs">
									<span class="flex items-center gap-2">
										<span class="inline-block size-2 rounded-full {opt.dot}"></span>
										{opt.label}
									</span>
								</DropdownMenu.RadioItem>
							{/each}
						</DropdownMenu.RadioGroup>
						<DropdownMenu.Separator />
						{#if connection}
							<DropdownMenu.CheckboxItem
								checked={connection.preserve_mass}
								onCheckedChange={togglePreserveMass}
								class="text-xs"
							>
								Preserve mass
							</DropdownMenu.CheckboxItem>
							<DropdownMenu.Separator />
						{/if}
					{/if}
					<DropdownMenu.Item
						class="text-xs text-destructive focus:text-destructive"
						onclick={remove}
					>
						<TrashIcon class="mr-2 size-3.5" />
						Delete Signature
					</DropdownMenu.Item>
				</DropdownMenu.Content>
			</DropdownMenu.Root>
		{/if}
	</div>
</div>
