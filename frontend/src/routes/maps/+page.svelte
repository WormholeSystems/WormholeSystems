<script lang="ts">
	// Every map you can reach, ordered by when the chain last changed: the map you want is
	// nearly always the one you were just in.
	import ArchiveIcon from '@lucide/svelte/icons/archive';
	import PinIcon from '@lucide/svelte/icons/pin';
	import PinOffIcon from '@lucide/svelte/icons/pin-off';
	import ArchiveRestoreIcon from '@lucide/svelte/icons/archive-restore';
	import MoreVerticalIcon from '@lucide/svelte/icons/more-vertical';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import SearchIcon from '@lucide/svelte/icons/search';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import UploadIcon from '@lucide/svelte/icons/upload';

	import { createMutation, createQuery } from '@tanstack/svelte-query';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';

	import { api, errorMessage, type TransferSectionId } from '$lib/api/client';
	import { after, apiAction } from '$lib/api/mutations';
	import { key, q } from '$lib/api/queries';
	import type { CharacterSummary } from '$lib/api/types/CharacterSummary';
	import type { CreateMapBody } from '$lib/api/types/CreateMapBody';
	import type { MapEntry } from '$lib/api/types/MapEntry';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import * as Field from '$lib/components/ui/field';
	import { Input } from '$lib/components/ui/input';
	import { Textarea } from '$lib/components/ui/textarea';
	import { timeAgo } from '$lib/format';
	import { lookup } from '$lib/lookup';
	import { ROLE_TONE } from '$lib/map/roles';
	import { createUserSettingsWriter } from '$lib/map/user-settings';
	import { TRANSFER_SECTIONS, readExportFile, type ExportFilePeek } from '$lib/map/transfer';
	import { ticking } from '$lib/now.svelte';
	import { filterMaps, splitArchived, totalsOf } from './page-model';
	import { cn } from '$lib/utils';

	let { data }: { data: { maps: MapEntry[]; me: CharacterSummary | null } } = $props();

	// An instance can limit map creation to named affiliations. Off the list, the create and
	// import buttons are left out rather than answering with a 403.
	const canCreate = $derived(data.me?.can_create_map ?? false);

	// The load's list is the first frame; after that the cache owns it, and the top bar's
	// pinned shortcuts read the same entry.
	const mapsQuery = createQuery(() => ({ ...q.myMaps(), initialData: data.maps }));
	const maps = $derived(mapsQuery.data);
	let query = $state('');
	let showArchived = $state(false);
	// "4m ago" has to stay true while the page is open.
	const clock = ticking(30_000);
	const now = $derived(clock.current);

	let creating = $state(false);
	let newName = $state('');
	let newDescription = $state('');

	// The whole entry, not just an id, so the dialog can name what it is about to destroy.
	let deleting = $state<MapEntry | null>(null);

	// Importing an export file as a new map.
	let importing = $state(false);
	let importFileInput = $state<HTMLInputElement | null>(null);
	let importPicked = $state<ExportFilePeek | null>(null);
	let importName = $state('');
	let importSections = $state<Partial<Record<TransferSectionId, boolean>>>({});

	const split = $derived(splitArchived(filterMaps(maps, query)));
	const active = $derived(split.active);
	const archived = $derived(split.archived);
	const totals = $derived(totalsOf(active));

	// Needs the created map's id for the navigation, so it is its own mutation rather
	// than an apiAction.
	const createMut = createMutation(() => ({
		mutationFn: (vars: CreateMapBody) => api.createMap(vars),
		onSuccess: async (map) => {
			creating = false;
			newName = '';
			newDescription = '';
			await goto(`/maps/${map.id}`);
		},
		onError: (err: unknown) => toast.error(errorMessage(err)),
	}));

	function create() {
		const name = newName.trim();
		if (!name || createMut.isPending) return;
		createMut.mutate({ name, description: newDescription.trim() || undefined });
	}

	async function pickImportFile(files: FileList | null) {
		importPicked = null;
		const file = files?.[0];
		if (!file) return;
		try {
			const peek = await readExportFile(file);
			importPicked = peek;
			importName = peek.mapName;
			importSections = Object.fromEntries(peek.sections.map((id) => [id, true]));
		} catch (err) {
			toast.error(errorMessage(err));
			if (importFileInput) importFileInput.value = '';
		}
	}

	// Connections and signatures hang off placed systems, so a fresh map cannot take them
	// without the systems section.
	const needsSystems = (id: TransferSectionId) => id === 'connections' || id === 'signatures';
	const importChosen = $derived.by(() => {
		if (!importPicked) return [];
		const chosen = importPicked.sections.filter((id) => importSections[id]);
		return chosen.includes('solarsystems') ? chosen : chosen.filter((id) => !needsSystems(id));
	});

	const importMut = createMutation(() => ({
		mutationFn: (vars: { name?: string; sections: TransferSectionId[]; content: string }) =>
			api.importMapAsNew(vars),
		onSuccess: async (map) => {
			importing = false;
			importPicked = null;
			await goto(`/maps/${map.id}`);
		},
		onError: (err: unknown) => toast.error(errorMessage(err)),
	}));

	function runImport() {
		const file = importPicked;
		if (!file || importChosen.length === 0 || importMut.isPending) return;
		importMut.mutate({
			name: importName.trim() || undefined,
			sections: importChosen,
			content: file.content,
		});
	}

	// The list and the top bar's shortcuts read the same cache entry, so one invalidation
	// does both.
	const act = apiAction(() => [key.maps]);

	const settingsWriter = createUserSettingsWriter();

	function setPinned(map: MapEntry, value: boolean) {
		after(settingsWriter.saveAsync(map.id, { is_pinned: value }), () =>
			toast.success(value ? `${map.name} pinned to the top bar` : `${map.name} unpinned`),
		);
	}

	function setArchived(map: MapEntry, value: boolean) {
		settingsWriter.save(map.id, { is_archived: value });
	}

	function confirmDelete() {
		const map = deleting;
		if (!map) return;
		deleting = null;
		act.mutate(() => api.deleteMap(map.id));
	}
</script>

{#snippet card(map: MapEntry)}
	<div
		class={cn(
			'group relative flex flex-col border border-border bg-card transition-colors hover:border-foreground/25',
			map.is_archived && 'opacity-60',
		)}
		data-testid="map-card"
		data-map={map.name}
	>
		<a href="/maps/{map.id}" class="flex flex-1 flex-col gap-2 p-3">
			<span class="flex items-start justify-between gap-2">
				<span class="min-w-0">
					<span class="block truncate font-heading text-sm font-semibold">{map.name}</span>
					{#if map.description}
						<span class="mt-0.5 block truncate text-xs text-muted-foreground">
							{map.description}
						</span>
					{/if}
				</span>
			</span>

			<span
				class="mt-auto flex items-center gap-3 font-mono text-[10px] tracking-wider text-muted-foreground uppercase"
			>
				<span class="tabular-nums">{map.system_count} sys</span>
				<span class="tabular-nums">{map.connection_count} conn</span>
				<span class="tabular-nums"
					>{map.member_count} member{map.member_count === 1 ? '' : 's'}</span
				>
			</span>
		</a>

		<div class="flex items-center justify-between border-t border-border/50 px-3 py-1.5">
			<span class="flex items-center gap-2">
				<Badge
					variant="outline"
					class={cn('h-4 px-1 text-[10px] uppercase', lookup(ROLE_TONE, map.role))}
				>
					{map.role}
				</Badge>
				{#if map.pilots_online > 0}
					<span
						class="flex items-center gap-1 font-mono text-[10px] text-emerald-500"
						data-testid="map-pilots"
					>
						<span class="size-1.5 animate-pulse rounded-full bg-emerald-500"></span>
						{map.pilots_online}
					</span>
				{/if}
			</span>

			<span class="flex items-center gap-2">
				<span class="font-mono text-[10px] whitespace-nowrap text-muted-foreground/60">
					{map.last_activity ? timeAgo(map.last_activity, now) : 'untouched'}
				</span>
				<DropdownMenu.Root>
					<DropdownMenu.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="ghost"
								size="icon"
								class="size-5"
								aria-label="Actions for {map.name}"
								data-testid="map-menu"
							>
								<MoreVerticalIcon />
							</Button>
						{/snippet}
					</DropdownMenu.Trigger>
					<DropdownMenu.Content align="end">
						<DropdownMenu.Group>
							<DropdownMenu.Item onSelect={() => goto(`/maps/${map.id}/settings`)}>
								<SettingsIcon />
								Settings
							</DropdownMenu.Item>
							<DropdownMenu.Item
								onSelect={() => setPinned(map, !map.is_pinned)}
								data-testid="map-pin"
							>
								{#if map.is_pinned}
									<PinOffIcon />
									Unpin from the top bar
								{:else}
									<PinIcon />
									Pin to the top bar
								{/if}
							</DropdownMenu.Item>
							<DropdownMenu.Item
								onSelect={() => setArchived(map, !map.is_archived)}
								data-testid="map-archive"
							>
								{#if map.is_archived}
									<ArchiveRestoreIcon />
									Unarchive
								{:else}
									<ArchiveIcon />
									Archive
								{/if}
							</DropdownMenu.Item>
							{#if map.role === 'owner'}
								<DropdownMenu.Separator />
								<DropdownMenu.Item
									class="text-destructive data-highlighted:text-destructive"
									onSelect={() => (deleting = map)}
									data-testid="map-delete"
								>
									<Trash2Icon />
									Delete
								</DropdownMenu.Item>
							{/if}
						</DropdownMenu.Group>
					</DropdownMenu.Content>
				</DropdownMenu.Root>
			</span>
		</div>
	</div>
{/snippet}

<div class="flex flex-col gap-4 p-6">
	<div class="flex flex-wrap items-end justify-between gap-3">
		<div>
			<h1 class="font-heading text-lg font-semibold tracking-tight">Maps</h1>
			<p class="mt-0.5 text-sm text-muted-foreground">
				Every chain you can reach, most recently flown first.
			</p>
		</div>
		{#if canCreate}
			<div class="flex items-center gap-2">
				<Button variant="outline" onclick={() => (importing = true)} data-testid="import-new-map">
					<UploadIcon data-icon="inline-start" />
					Import
				</Button>
				<Button onclick={() => (creating = true)} data-testid="new-map">
					<PlusIcon data-icon="inline-start" />
					New map
				</Button>
			</div>
		{/if}
	</div>

	{#if maps.length > 0}
		<div class="flex w-fit divide-x divide-border border border-border">
			{#each [{ label: 'Maps', value: totals.maps }, { label: 'Systems', value: totals.systems }, { label: 'Pilots online', value: totals.pilots }] as stat (stat.label)}
				<div class="min-w-28 px-4 py-2">
					<div class="font-mono text-lg tabular-nums">{stat.value}</div>
					<div class="font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
						{stat.label}
					</div>
				</div>
			{/each}
		</div>

		<div class="flex flex-wrap items-center gap-2">
			<div class="relative max-w-md flex-1">
				<SearchIcon class="absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
				<Input bind:value={query} placeholder="Search maps" class="pl-9" data-testid="map-search" />
			</div>
			{#if archived.length > 0}
				<Button
					variant="outline"
					onclick={() => (showArchived = !showArchived)}
					data-testid="toggle-archived"
				>
					{showArchived ? 'Hide' : 'Show'} archived ({archived.length})
				</Button>
			{/if}
		</div>
	{/if}

	{#if maps.length === 0}
		<div class="flex flex-col items-start gap-3 border border-dashed border-border p-8">
			{#if canCreate}
				<p class="text-sm text-muted-foreground" data-testid="maps-empty">
					No maps yet. A map is one chain: create one and start scanning.
				</p>
				<Button onclick={() => (creating = true)}>
					<PlusIcon data-icon="inline-start" />
					Create your first map
				</Button>
			{:else}
				<p class="text-sm text-muted-foreground" data-testid="maps-empty">
					No maps yet. On this instance only certain characters, corporations or alliances create
					maps; once one of them adds you to theirs, it shows up here.
				</p>
			{/if}
		</div>
	{:else if active.length === 0}
		<!-- Told apart from having no maps at all, because the fix is different. -->
		<div class="flex flex-col items-start gap-3 border border-dashed border-border p-8">
			<p class="text-sm text-muted-foreground" data-testid="maps-none-active">
				{query.trim()
					? `No active maps match "${query.trim()}".`
					: 'Every map you can reach is archived.'}
			</p>
			{#if archived.length > 0 && !showArchived}
				<Button variant="outline" onclick={() => (showArchived = true)}>
					Show archived ({archived.length})
				</Button>
			{/if}
		</div>
	{:else}
		<div class="grid gap-3" style="grid-template-columns: repeat(auto-fill, minmax(18rem, 1fr))">
			{#each active as map (map.id)}
				{@render card(map)}
			{/each}
		</div>
	{/if}

	{#if showArchived && archived.length > 0}
		<div class="flex items-center gap-3 pt-2">
			<span class="font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
				Archived · {archived.length}
			</span>
			<span class="h-px flex-1 bg-border"></span>
		</div>
		<div class="grid gap-3" style="grid-template-columns: repeat(auto-fill, minmax(18rem, 1fr))">
			{#each archived as map (map.id)}
				{@render card(map)}
			{/each}
		</div>
	{/if}
</div>

<Dialog.Root bind:open={creating}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>Create a map</Dialog.Title>
			<Dialog.Description>
				One map is one chain. Access and naming can be set up afterwards.
			</Dialog.Description>
		</Dialog.Header>
		<form
			onsubmit={(e) => {
				e.preventDefault();
				create();
			}}
		>
			<Field.FieldGroup>
				<Field.Field>
					<Field.FieldLabel for="map-name">Name</Field.FieldLabel>
					<Input
						id="map-name"
						bind:value={newName}
						placeholder="Home chain"
						autofocus
						data-testid="new-map-name"
					/>
				</Field.Field>
				<Field.Field>
					<Field.FieldLabel for="map-description">Description</Field.FieldLabel>
					<Textarea
						id="map-description"
						bind:value={newDescription}
						placeholder="Optional. What this chain is for."
						rows={2}
						data-testid="new-map-description"
					/>
				</Field.Field>
			</Field.FieldGroup>
			<Dialog.Footer class="mt-4">
				<Button type="button" variant="ghost" onclick={() => (creating = false)}>Cancel</Button>
				<Button
					type="submit"
					disabled={!newName.trim() || createMut.isPending}
					data-testid="new-map-create"
				>
					Create
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<Dialog.Root bind:open={importing}>
	<Dialog.Content class="sm:max-w-md" data-testid="import-map-dialog">
		<Dialog.Header>
			<Dialog.Title>Import a map</Dialog.Title>
			<Dialog.Description>
				Create a fresh map from an export file, yours to own. Files from legacy wormholesystems work
				too.
			</Dialog.Description>
		</Dialog.Header>
		<div class="flex flex-col gap-4">
			<input
				bind:this={importFileInput}
				type="file"
				accept=".json,application/json"
				class="text-sm text-muted-foreground file:mr-3 file:border file:border-border file:bg-secondary file:px-3 file:py-1.5 file:text-sm file:text-secondary-foreground hover:file:bg-secondary/80"
				onchange={(e) => pickImportFile(e.currentTarget.files)}
				data-testid="import-map-file"
			/>

			{#if importPicked}
				<Field.Field>
					<Field.FieldLabel for="import-map-name">Name</Field.FieldLabel>
					<Input
						id="import-map-name"
						bind:value={importName}
						placeholder={importPicked.mapName}
						data-testid="import-map-name"
					/>
				</Field.Field>

				<div class="flex flex-col gap-1">
					<span class="text-sm font-medium">Sections</span>
					{#each TRANSFER_SECTIONS.filter( (s) => importPicked?.sections.includes(s.id) ) as section (section.id)}
						{@const blocked =
							needsSystems(section.id) && !(importSections['solarsystems'] ?? false)}
						<label
							class={cn('flex items-center gap-2 py-1 text-sm', blocked && 'text-muted-foreground')}
						>
							<input
								type="checkbox"
								class="accent-primary"
								checked={(importSections[section.id] ?? false) && !blocked}
								disabled={blocked}
								onchange={(e) => (importSections[section.id] = e.currentTarget.checked)}
							/>
							{section.label}
							{#if blocked}
								<span class="text-xs">(needs Systems)</span>
							{/if}
						</label>
					{/each}
				</div>
			{/if}
		</div>
		<Dialog.Footer class="mt-4">
			<Button type="button" variant="ghost" onclick={() => (importing = false)}>Cancel</Button>
			<Button
				onclick={runImport}
				disabled={!importPicked || importChosen.length === 0 || importMut.isPending}
				data-testid="import-map-create"
			>
				Import
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>

<Dialog.Root open={deleting !== null} onOpenChange={(o) => !o && (deleting = null)}>
	<Dialog.Content class="sm:max-w-md" data-testid="delete-map-dialog">
		<Dialog.Header>
			<Dialog.Title>Delete “{deleting?.name}”?</Dialog.Title>
			<Dialog.Description>
				This removes the map and everything on it for everyone who can see it. There is no undo.
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer>
			<Button variant="ghost" onclick={() => (deleting = null)}>Cancel</Button>
			<Button variant="destructive" onclick={confirmDelete} data-testid="confirm-delete-map">
				Delete map
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
