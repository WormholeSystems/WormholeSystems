<script lang="ts">
	// Signatures for the active system: sortable columns, category filters, clipboard paste
	// with diff tints and lazy delete, and per-row editing.
	import ClipboardPasteIcon from '@lucide/svelte/icons/clipboard-paste';
	import LayersIcon from '@lucide/svelte/icons/layers';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import Rows2Icon from '@lucide/svelte/icons/rows-2';
	import Rows3Icon from '@lucide/svelte/icons/rows-3';
	import TrashIcon from '@lucide/svelte/icons/trash-2';
	import * as v from 'valibot';
	import { readStored, writeStored } from '$lib/storage';
	import { sortState } from '$lib/sort-state.svelte';
	import { tickingMs } from '$lib/now.svelte';

	import { toast } from 'svelte-sonner';

	import { readText } from '$lib/clipboard';
	import { systemResolver } from '$lib/resolve-cache.svelte';
	import type { MapSystemView } from '$lib/api/types/MapSystemView';
	import type { PastedSignature } from '$lib/api/types/PastedSignature';
	import type { Signature } from '$lib/api/types/Signature';
	import type { UpdateMapUserSettings } from '$lib/api/types/UpdateMapUserSettings';
	import { createQuery } from '@tanstack/svelte-query';

	import { q } from '$lib/api/queries';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Separator } from '$lib/components/ui/separator';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import MapPanel from '$lib/components/map-panel/MapPanel.svelte';
	import MapPanelContent from '$lib/components/map-panel/MapPanelContent.svelte';
	import MapPanelHeader from '$lib/components/map-panel/MapPanelHeader.svelte';
	import { cn } from '$lib/utils';
	import {
		CATEGORIES,
		compareSignatures,
		formatSignatureId,
		loadScanNames,
		parseScan,
		typeById,
	} from '$lib/map/signatures';
	import type { MapState } from '../../state/map-state.svelte';
	import MismatchDialog from './MismatchDialog.svelte';
	import { makeSignatureContext } from '$lib/map/signature-context';
	import { deletedByPaste, pasteStatus } from '$lib/signatures/paste-diff';
	import SignatureColumns, { SORT_COLUMNS } from '$lib/components/SignatureColumns.svelte';
	import SignatureRow from '$lib/components/SignatureRow.svelte';
	import SignaturesPastedAt from './SignaturesPastedAt.svelte';
	import { solarSystemId, systemName } from '$lib/map/system';

	let {
		map,
		system,
	}: {
		map: MapState;
		system: MapSystemView;
	} = $props();

	// svelte-ignore state_referenced_locally -- the map instance is stable for this mount.
	const ctx = makeSignatureContext(map);
	// One clock for every row's age, rather than a timer per row. The ages count whole
	// minutes, so half a minute is plenty.
	const clock = tickingMs(30_000);

	const catalogQuery = createQuery(() => q.signatureCatalog());
	const catalog = $derived(catalogQuery.data ?? null);

	// A ghost has no system to scan against, so the panel says so instead of offering a
	// paste box that the server would refuse.
	const systemId = $derived(solarSystemId(system));
	const canWrite = $derived(map.canWrite && systemId !== null);
	const compact = $derived(map.userSettings?.compact_signature_list ?? false);
	const targetLabel = $derived.by(() => {
		const name = systemName(system);
		if (system.alias && name) return `${system.alias} (${name})`;
		return name ?? system.alias ?? 'this system';
	});
	const showStaticsFirst = $derived(map.userSettings?.show_statics_first ?? false);

	// The HIDDEN set is what persists, so a new category defaults to visible.
	let hidden = $state<string[]>(
		readStored('signatures-category-hidden-filters', v.array(v.string()), []),
	);
	$effect(() => {
		writeStored('signatures-category-hidden-filters', hidden);
	});
	// Default: id desc, ties by id ascending, nulls last.
	const sort = sortState('signatures-sort', SORT_COLUMNS, { column: 'id', direction: 'desc' });

	const mySigs = $derived(map.signatures.all.filter((s) => s.solar_system_id === systemId));
	const filtered = $derived(mySigs.filter((s) => !hidden.includes(s.group)));
	const hiddenCount = $derived(mySigs.length - filtered.length);

	function typeName(s: Signature): string | null {
		return (catalog && typeById(catalog, s.signature_type_id)?.name) ?? s.name;
	}

	const sorted = $derived.by(() => {
		const dir = sort.current.direction === 'asc' ? 1 : -1;
		return filtered.toSorted((a, b) => {
			const cmp = compareSignatures(a, b, sort.current.column, typeName);
			if (cmp !== 0) return cmp * dir;
			return a.signature_id.localeCompare(b.signature_id);
		});
	});

	// The pre-paste ids are snapshotted so new (green) against updated (amber) stays stable
	// after the round-trip creates the new rows.
	let pasted = $state<PastedSignature[] | null>(null);
	let preIds = $state<Set<string>>(new Set());
	let pending = $state<PastedSignature[] | null>(null);
	let mismatchOpen = $state(false);
	let mismatchSystem = $state('Unknown');

	// A refetch hands the panel a fresh `system` object, so guard on the id: the paste
	// selection only clears when the active system really changes.
	let lastSystemId = $state<number | null>(null);
	$effect(() => {
		if (system.id !== lastSystemId) {
			lastSystemId = system.id;
			pasted = null;
			pending = null;
		}
	});

	const pastedIds = $derived(pasted === null ? null : new Set(pasted.map((p) => p.signature_id)));
	const deletedSigs = $derived(deletedByPaste(mySigs, pastedIds));
	function rowStatus(s: Signature): 'new' | 'updated' | 'deleted' | null {
		return pasteStatus(s, preIds, pastedIds);
	}

	async function handlePasteText(text: string) {
		if (!canWrite) return;
		const [types, names] = await Promise.all([map.loadCatalog(), loadScanNames()]);
		const rows = parseScan(text, types, names);
		if (rows.length === 0) {
			toast.error('Nothing in that paste looked like a signature');
			return;
		}
		const active = map.characters.mine.find((c) => c.is_active);
		if (active?.solar_system_id != null && active.solar_system_id !== systemId) {
			pending = rows;
			systemResolver
				.resolve(active.solar_system_id)
				.then((hit) => (mismatchSystem = hit?.name ?? 'Unknown'))
				.finally(() => (mismatchOpen = true));
			return;
		}
		commitPaste(rows);
	}

	function commitPaste(rows: PastedSignature[]) {
		if (systemId === null) return;
		preIds = new Set(mySigs.map((s) => s.signature_id));
		pasted = rows;
		map.signatures.paste(systemId, rows);
	}

	function onWindowPaste(e: ClipboardEvent) {
		if (!canWrite) return;
		const el = e.target instanceof HTMLElement ? e.target : null;
		if (el?.closest('input, textarea, [contenteditable]')) return;
		const text = e.clipboardData?.getData('text/plain');
		if (!text) return;
		e.preventDefault();
		handlePasteText(text);
	}

	async function pasteFromClipboard() {
		const text = await readText();
		if (text !== null) handlePasteText(text);
	}

	function deleteMissing() {
		map.signatures.removeMissing(deletedSigs.map((s) => s.id));
		pasted = null;
	}

	// The inline new row saves itself once a valid 7-character id is typed.
	let creating = $state(false);
	let newId = $state('');
	let newInput = $state<HTMLInputElement | null>(null);

	function startCreate() {
		creating = true;
		newId = '';
		setTimeout(() => newInput?.focus());
	}

	function saveNew() {
		const value = newId.trim();
		creating = false;
		newId = '';
		if (value.length === 7 && systemId !== null) {
			map.signatures.add(systemId, value);
		}
	}

	function setSetting(patch: UpdateMapUserSettings) {
		map.patchUserSettings(patch);
	}
</script>

<svelte:window onpaste={onWindowPaste} />

<MapPanel testid="signatures-card">
	<MapPanelHeader>
		Signatures
		{#if filtered.length > 0}
			<span class="ml-1 text-amber-400">{filtered.length}</span>
		{/if}
		{#if hiddenCount > 0}
			<span class="ml-1 text-muted-foreground/70">{hiddenCount} hidden</span>
		{/if}
		{#if system.kind === 'system'}
			<SignaturesPastedAt pastedAt={system.signatures_pasted_at} />
		{/if}
		{#snippet actions()}
			<Tooltip.Provider delayDuration={300}>
				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="ghost"
								size="icon"
								class="size-6 text-muted-foreground hover:text-foreground"
								aria-label={compact
									? 'Switch to comfortable signature list'
									: 'Switch to compact signature list'}
								aria-pressed={compact}
								data-testid="compact-toggle"
								onclick={() => setSetting({ compact_signature_list: !compact })}
							>
								{#if compact}
									<Rows2Icon class="size-3.5" />
								{:else}
									<Rows3Icon class="size-3.5" />
								{/if}
							</Button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content>
						{compact ? 'Comfortable rows' : 'Compact rows'}
					</Tooltip.Content>
				</Tooltip.Root>
				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="ghost"
								size="icon"
								class={cn(
									'relative size-6',
									showStaticsFirst
										? 'text-foreground hover:text-foreground'
										: 'text-muted-foreground hover:text-foreground',
								)}
								aria-pressed={showStaticsFirst}
								aria-label="Statics first in the wormhole type picker"
								data-testid="statics-first-toggle"
								onclick={() => setSetting({ show_statics_first: !showStaticsFirst })}
							>
								<LayersIcon class="size-3.5" />
								{#if showStaticsFirst}
									<span class="absolute inset-x-1.5 bottom-0.5 h-px bg-current"></span>
								{/if}
							</Button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content>
						Statics first in the type picker: {showStaticsFirst ? 'on' : 'off'}
					</Tooltip.Content>
				</Tooltip.Root>
				<Separator orientation="vertical" class="mx-0.5 data-[orientation=vertical]:h-4" />
				<div class="flex items-center gap-0.5">
					{#each CATEGORIES as c (c.group)}
						{@const shown = !hidden.includes(c.group)}
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<Button
										{...props}
										variant="ghost"
										size="icon"
										class={cn(
											'size-6',
											shown ? c.color : 'text-muted-foreground/40 hover:text-muted-foreground',
										)}
										aria-pressed={shown}
										aria-label={c.label}
										data-testid="filter-{c.group}"
										onclick={() => {
											hidden = shown ? [...hidden, c.group] : hidden.filter((g) => g !== c.group);
										}}
									>
										<c.icon class="size-3.5" />
									</Button>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>{shown ? `Hide ${c.label}` : `Show ${c.label}`}</Tooltip.Content>
						</Tooltip.Root>
					{/each}
				</div>
				<Separator orientation="vertical" class="mx-0.5 data-[orientation=vertical]:h-4" />
				{#if canWrite}
					{#if pasted !== null}
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<Button
										{...props}
										variant="ghost"
										class="h-6 px-2 text-[11px] leading-none"
										onclick={() => (pasted = null)}
									>
										Unselect
									</Button>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>Drop the highlight from the last paste</Tooltip.Content>
						</Tooltip.Root>
					{/if}
					{#if deletedSigs.length > 0}
						<Tooltip.Root>
							<Tooltip.Trigger>
								{#snippet child({ props })}
									<Button
										{...props}
										variant="destructive"
										size="icon"
										class="size-6"
										aria-label="Delete missing signatures"
										data-testid="delete-missing"
										onclick={deleteMissing}
									>
										<TrashIcon class="size-3.5" />
									</Button>
								{/snippet}
							</Tooltip.Trigger>
							<Tooltip.Content>
								Delete the {deletedSigs.length} missing and their connections
							</Tooltip.Content>
						</Tooltip.Root>
					{/if}
					<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<Button
									{...props}
									variant="ghost"
									size="icon"
									class="size-6 text-muted-foreground hover:text-foreground"
									aria-label="Paste signatures"
									data-testid="paste-clipboard"
									onclick={pasteFromClipboard}
								>
									<ClipboardPasteIcon class="size-3.5" />
								</Button>
							{/snippet}
						</Tooltip.Trigger>
						<Tooltip.Content>Paste from the clipboard (Ctrl/Cmd + V)</Tooltip.Content>
					</Tooltip.Root>
					<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<Button
									{...props}
									variant="ghost"
									size="icon"
									class="size-6 text-muted-foreground hover:text-foreground"
									aria-label="Create new signature"
									data-testid="new-signature"
									onclick={startCreate}
								>
									<PlusIcon class="size-3.5" />
								</Button>
							{/snippet}
						</Tooltip.Trigger>
						<Tooltip.Content>New signature</Tooltip.Content>
					</Tooltip.Root>
				{/if}
			</Tooltip.Provider>
		{/snippet}
	</MapPanelHeader>
	<MapPanelContent>
		{#if system.kind === 'ghost'}
			<div class="flex flex-col items-center justify-center gap-2 p-4 text-center">
				<p class="max-w-56 text-[11px] text-muted-foreground">
					An unmapped hole has nothing to scan yet. Assign a system to it and its signatures land
					here.
				</p>
			</div>
		{:else}
			<Tooltip.Provider delayDuration={300}>
				<SignatureColumns {compact} sort={sort.current} onsort={sort.toggle} />

				{#if creating}
					<div
						class="flex items-center gap-2 border-b border-border/30 px-3 {compact
							? 'py-0.5'
							: 'py-1.5'}"
					>
						<div class="w-16 shrink-0">
							<Input
								bind:ref={newInput}
								value={newId}
								oninput={(e) => (newId = formatSignatureId(e.currentTarget.value))}
								onblur={saveNew}
								onkeydown={(e) => {
									if (e.key === 'Enter') saveNew();
									if (e.key === 'Escape') {
										creating = false;
										newId = '';
									}
								}}
								class="w-full rounded-none border-border/50 bg-background/50 px-1.5 font-mono text-xs uppercase {compact
									? 'h-5'
									: 'h-6'}"
								maxlength={7}
								placeholder="XXX-XXX"
								data-testid="new-signature-id"
							/>
						</div>
						<span class="text-xs text-muted-foreground">New signature</span>
					</div>
				{/if}

				{#if catalog && sorted.length > 0}
					{#each sorted as sig (sig.id)}
						<SignatureRow
							{ctx}
							{system}
							{sig}
							{catalog}
							{compact}
							{canWrite}
							{showStaticsFirst}
							status={rowStatus(sig)}
							now={clock.current}
						/>
					{/each}
				{:else if !creating}
					<div class="flex flex-col items-center justify-center gap-2 p-4">
						<p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">
							{hiddenCount > 0 ? `${hiddenCount} hidden by filters` : 'No signatures'}
						</p>
					</div>
				{/if}
			</Tooltip.Provider>
		{/if}
	</MapPanelContent>
</MapPanel>

<MismatchDialog
	bind:open={mismatchOpen}
	{targetLabel}
	characterSystem={mismatchSystem}
	onconfirm={() => {
		if (pending !== null) commitPaste(pending);
		pending = null;
	}}
	oncancel={() => (pending = null)}
/>
