<script lang="ts">
	// The map as a file: download the sections you pick, or merge a file into this map.
	// The format is the legacy wormholesystems export, so files move between the two
	// applications in both directions. Manager+, like the legacy transfer page.
	import DownloadIcon from '@lucide/svelte/icons/download';
	import UploadIcon from '@lucide/svelte/icons/upload';

	import { createQuery } from '@tanstack/svelte-query';
	import { toast } from 'svelte-sonner';

	import { api, errorMessage } from '$lib/api/client';
	import { apiAction } from '$lib/api/mutations';
	import { key, q } from '$lib/api/queries';
	import type { ImportSummary } from '$lib/api/types/ImportSummary';
	import type { SectionCounts } from '$lib/api/types/SectionCounts';
	import {
		TRANSFER_SECTIONS,
		readExportFile,
		type ExportFilePeek,
		type TransferSectionId,
	} from '$lib/map/transfer';
	import SettingRow from '$lib/components/settings/SettingRow.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { Switch } from '$lib/components/ui/switch';
	import { mapSettings } from '../context.svelte';

	const ctx = mapSettings();
	const mapId = $derived(ctx.mapId);
	const countsQuery = createQuery(() => q.transferCounts(mapId));
	const counts = $derived(countsQuery.data);

	function countFor(section: TransferSectionId): number | null {
		if (!counts) return null;
		switch (section) {
			case 'settings':
				return null;
			case 'access':
				return counts.access;
			case 'solarsystems':
				return counts.systems;
			case 'connections':
				return counts.connections;
			case 'signatures':
				return counts.signatures;
			case 'routes':
				return counts.routes;
		}
	}

	// --- Export ---

	let exportSections = $state<Record<TransferSectionId, boolean>>({
		settings: true,
		access: true,
		solarsystems: true,
		connections: true,
		signatures: true,
		routes: true,
	});
	const exportChosen = $derived(TRANSFER_SECTIONS.filter((s) => exportSections[s.id]));

	function download() {
		location.assign(
			api.exportMapUrl(
				mapId,
				exportChosen.map((s) => s.id),
			),
		);
	}

	// --- Import ---

	let fileInput = $state<HTMLInputElement | null>(null);
	let picked = $state<ExportFilePeek | null>(null);
	let importSections = $state<Partial<Record<TransferSectionId, boolean>>>({});
	let summary = $state<ImportSummary | null>(null);

	async function pickFile(files: FileList | null) {
		summary = null;
		picked = null;
		const file = files?.[0];
		if (!file) return;
		try {
			const peek = await readExportFile(file);
			picked = peek;
			importSections = Object.fromEntries(peek.sections.map((id) => [id, true]));
		} catch (err) {
			toast.error(errorMessage(err));
			if (fileInput) fileInput.value = '';
		}
	}

	const importChosen = $derived(picked ? picked.sections.filter((id) => importSections[id]) : []);

	// An import can change nearly everything, so the whole map subtree refetches.
	const importAct = apiAction(() => [key.map(mapId), key.maps]);

	function runImport() {
		const file = picked;
		if (!file || importChosen.length === 0) return;
		importAct.mutate(async () => {
			summary = await api.importMap(mapId, importChosen, file.content);
			toast.success('Import complete.');
		});
	}

	const SUMMARY_ROWS: {
		id: TransferSectionId;
		label: string;
		pick: (s: ImportSummary) => SectionCounts;
	}[] = [
		{ id: 'settings', label: 'Settings', pick: (s) => s.settings },
		{ id: 'access', label: 'Access', pick: (s) => s.access },
		{ id: 'solarsystems', label: 'Systems', pick: (s) => s.systems },
		{ id: 'connections', label: 'Connections', pick: (s) => s.connections },
		{ id: 'signatures', label: 'Signatures', pick: (s) => s.signatures },
		{ id: 'routes', label: 'Routes', pick: (s) => s.routes },
	];
</script>

<div class="flex flex-col gap-6">
	<Card.Root>
		<Card.Header>
			<Card.Title>Export</Card.Title>
			<Card.Description>
				Download the selected sections as a JSON file. The share link, webhooks, personal settings
				and the owner grant never leave the map. The file also imports into legacy wormholesystems.
			</Card.Description>
		</Card.Header>
		<Card.Content class="flex flex-col py-0">
			{#each TRANSFER_SECTIONS as section (section.id)}
				{@const count = countFor(section.id)}
				<SettingRow
					id={`export-${section.id}`}
					label={count === null ? section.label : `${section.label} (${count})`}
					description={section.description}
				>
					{#snippet control()}
						<Switch
							checked={exportSections[section.id]}
							aria-label={`Export ${section.label}`}
							onCheckedChange={(v) => (exportSections[section.id] = v)}
						/>
					{/snippet}
				</SettingRow>
			{/each}
		</Card.Content>
		<Card.Footer class="justify-end">
			<Button onclick={download} disabled={exportChosen.length === 0} data-testid="export-map">
				<DownloadIcon data-icon="inline-start" />
				Download export
			</Button>
		</Card.Footer>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Import into this map</Card.Title>
			<Card.Description>
				Merge an export file into this map. Existing systems, signatures and grants are updated in
				place; a connection that already exists is left alone. The owner grant is never touched. To
				start a fresh map from a file instead, use Import on the maps page.
			</Card.Description>
		</Card.Header>
		<Card.Content class="flex flex-col gap-4 pt-0">
			<input
				bind:this={fileInput}
				type="file"
				accept=".json,application/json"
				class="text-sm text-muted-foreground file:mr-3 file:border file:border-border file:bg-secondary file:px-3 file:py-1.5 file:text-sm file:text-secondary-foreground hover:file:bg-secondary/80"
				onchange={(e) => pickFile(e.currentTarget.files)}
				data-testid="import-file"
			/>

			{#if picked}
				<div class="flex flex-col">
					<p class="pb-1 text-xs text-muted-foreground">
						“{picked.mapName}”: pick what to bring in.
					</p>
					{#each TRANSFER_SECTIONS.filter( (s) => picked?.sections.includes(s.id) ) as section (section.id)}
						<SettingRow
							id={`import-${section.id}`}
							label={section.label}
							description={section.description}
						>
							{#snippet control()}
								<Switch
									checked={importSections[section.id] ?? false}
									aria-label={`Import ${section.label}`}
									onCheckedChange={(v) => (importSections[section.id] = v)}
								/>
							{/snippet}
						</SettingRow>
					{/each}
				</div>
			{/if}

			{#if summary}
				<div class="border border-border/60 p-3" data-testid="import-summary">
					<p class="pb-2 text-xs font-medium">What the import did</p>
					<div class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-xs text-muted-foreground">
						{#each SUMMARY_ROWS.filter((row) => importChosen.includes(row.id)) as row (row.id)}
							{@const c = row.pick(summary)}
							<span>{row.label}</span>
							<span class="tabular-nums">
								{c.created} created, {c.updated} updated, {c.skipped} skipped
							</span>
						{/each}
					</div>
				</div>
			{/if}
		</Card.Content>
		<Card.Footer class="justify-end">
			<Button
				onclick={runImport}
				disabled={!picked || importChosen.length === 0 || importAct.isPending}
				data-testid="import-map"
			>
				<UploadIcon data-icon="inline-start" />
				Import
			</Button>
		</Card.Footer>
	</Card.Root>
</div>
