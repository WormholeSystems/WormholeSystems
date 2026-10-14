<script lang="ts">
	// What the map shows you, per viewer. Placement is the exception: the mode is the map's,
	// and the row only appears when the map hands the choice to each viewer.
	import { createQuery } from '@tanstack/svelte-query';
	import ImageUpIcon from '@lucide/svelte/icons/image-up';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import { page } from '$app/state';
	import { createBackgroundImageWriter, createUserSettingsWriter } from '$lib/map/user-settings';
	import {
		ACCEPTED_IMAGE_TYPES,
		BACKGROUND_MODES,
		BACKGROUND_MODE_VALUES,
		takePickedFile,
	} from '$lib/map/background';
	import { KILLMAIL_FILTERS } from '$lib/map/killmails';
	import { PLACEMENTS as BASE_PLACEMENTS } from '$lib/map/placement';
	import { q } from '$lib/api/queries';
	import type { MapView } from '$lib/api/types/MapView';
	import type { UpdateMapUserSettings } from '$lib/api/types/UpdateMapUserSettings';
	import SettingRow from '$lib/components/settings/SettingRow.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import * as Select from '$lib/components/ui/select';
	import { Switch } from '$lib/components/ui/switch';
	import type { MapLayout } from '$lib/api/types/MapLayout';
	import { oneOf } from '$lib/lookup';

	let { data }: { data: { view: MapView } } = $props();

	const mapId = $derived(Number(page.params.id) || 0);
	const settingsQuery = createQuery(() => q.mapUserSettings(mapId));
	const settings = $derived(settingsQuery.data ?? null);
	const viewQuery = createQuery(() => ({ ...q.mapView(mapId), initialData: data.view }));
	const view = $derived(viewQuery.data);

	const settingsWriter = createUserSettingsWriter();
	const saveUserSettings = (patch: UpdateMapUserSettings) => settingsWriter.save(mapId, patch);

	const image = createBackgroundImageWriter();
	let fileInput = $state<HTMLInputElement | null>(null);

	function chooseImage(event: Event) {
		const file = takePickedFile(event);
		if (file) image.upload(mapId, file);
	}

	const backgroundMode = $derived(settings?.background_image_mode ?? 'grid');
	const backgroundImage = $derived(settings?.background_image_url ?? null);

	// `map` is not a layout, it is the absence of an override: this viewer follows whatever
	// the map itself is set to.
	const PLACEMENTS: readonly { value: MapLayout | 'map'; label: string }[] = [
		{ value: 'map', label: 'Follow the map' },
		...BASE_PLACEMENTS.map((o) => ({ value: o.value, label: o.label })),
	];
	const PLACEMENT_VALUES = PLACEMENTS.map((p) => p.value);
	const placement = $derived(settings?.layout_override ?? 'map');

	const FILTER_VALUES = KILLMAIL_FILTERS.map((f) => f.value);
	const filter = $derived(settings?.killmail_filter ?? 'all');
</script>

<Card.Root>
	<Card.Header>
		<Card.Title>Display</Card.Title>
		<Card.Description>How this map looks to you, and nobody else.</Card.Description>
	</Card.Header>
	<Card.Content class="flex flex-col py-0">
		<SettingRow
			id="show-threat-level"
			label="Threat level on nodes"
			description="Colours a wormhole by how much has died there lately. Useful when scouting somewhere new, noise once you know the chain."
		>
			{#snippet control()}
				<Switch
					checked={settings?.show_threat_level ?? true}
					aria-label="Threat level on nodes"
					onCheckedChange={(v) => saveUserSettings({ show_threat_level: v })}
				/>
			{/snippet}
		</SettingRow>

		<SettingRow
			id="show-statics-first"
			label="Statics first in the wormhole list"
			description="Puts a system's own statics at the top of the type picker, where they are what you are almost always looking for."
		>
			{#snippet control()}
				<Switch
					checked={settings?.show_statics_first ?? false}
					aria-label="Statics first in the wormhole list"
					onCheckedChange={(v) => saveUserSettings({ show_statics_first: v })}
				/>
			{/snippet}
		</SettingRow>

		<SettingRow
			id="compact-signature-list"
			label="Compact signature list"
			description="Tighter rows, so a freshly scanned system fits without scrolling."
		>
			{#snippet control()}
				<Switch
					checked={settings?.compact_signature_list ?? false}
					aria-label="Compact signature list"
					onCheckedChange={(v) => saveUserSettings({ compact_signature_list: v })}
				/>
			{/snippet}
		</SettingRow>

		{#if view?.map.allow_layout_override}
			<SettingRow
				id="layout-override"
				label="How this chain is placed"
				description="Custom placement is the map as people dragged it. Automatic draws it as a tree from the connections, which nobody can move. Following the map takes whichever it is set to."
			>
				{#snippet control()}
					<Select.Root
						type="single"
						value={placement}
						onValueChange={(v) => {
							const picked = oneOf(PLACEMENT_VALUES, v);
							if (picked) {
								saveUserSettings({
									layout_override: picked === 'map' ? null : picked,
								});
							}
						}}
					>
						<Select.Trigger class="w-52" data-testid="layout-override-select">
							{PLACEMENTS.find((p) => p.value === placement)?.label}
						</Select.Trigger>
						<Select.Content>
							<Select.Group>
								{#each PLACEMENTS as option (option.value)}
									<Select.Item value={option.value} label={option.label}>
										{option.label}
									</Select.Item>
								{/each}
							</Select.Group>
						</Select.Content>
					</Select.Root>
				{/snippet}
			</SettingRow>
		{/if}

		<SettingRow
			id="background-image"
			label="Background image"
			description="A picture behind the chain, for you only. PNG, JPEG, GIF or WebP, up to 8 MiB."
		>
			{#snippet control()}
				<input
					bind:this={fileInput}
					type="file"
					accept={ACCEPTED_IMAGE_TYPES}
					class="hidden"
					data-testid="background-image-input"
					onchange={chooseImage}
				/>
				{#if backgroundImage}
					<img
						src={backgroundImage}
						alt=""
						class="h-9 w-16 border border-border object-cover"
						data-testid="background-image-preview"
					/>
				{/if}
				<Button
					variant="outline"
					size="sm"
					disabled={image.pending}
					onclick={() => fileInput?.click()}
					data-testid="background-image-choose"
				>
					<ImageUpIcon data-icon="inline-start" />
					{backgroundImage ? 'Replace' : 'Choose image'}
				</Button>
				{#if backgroundImage}
					<Button
						variant="ghost"
						size="sm"
						class="text-muted-foreground hover:text-destructive"
						disabled={image.pending}
						onclick={() => image.remove(mapId)}
						aria-label="Remove background image"
						data-testid="background-image-remove"
					>
						<Trash2Icon data-icon="inline-start" />
						Remove
					</Button>
				{/if}
			{/snippet}
		</SettingRow>

		{#if backgroundImage}
			<SettingRow
				id="background-image-mode"
				label="How the picture sits"
				description="Moving with the map paints it across the whole grid, so it pans and zooms with the systems. Filling the panel keeps it still behind everything."
			>
				{#snippet control()}
					<Select.Root
						type="single"
						value={backgroundMode}
						onValueChange={(v) => {
							const picked = oneOf(BACKGROUND_MODE_VALUES, v);
							if (picked) saveUserSettings({ background_image_mode: picked });
						}}
					>
						<Select.Trigger class="w-52" data-testid="background-mode-select">
							{BACKGROUND_MODES.find((m) => m.value === backgroundMode)?.label}
						</Select.Trigger>
						<Select.Content>
							<Select.Group>
								{#each BACKGROUND_MODES as option (option.value)}
									<Select.Item value={option.value} label={option.label}>{option.label}</Select.Item
									>
								{/each}
							</Select.Group>
						</Select.Content>
					</Select.Root>
				{/snippet}
			</SettingRow>
		{/if}

		<SettingRow
			id="killmail-filter"
			label="Killmails to show"
			description="The card lists kills in the systems on this map. Narrow it to one half of the chain when the other half is drowning it out, or to whichever system you have selected."
		>
			{#snippet control()}
				<Select.Root
					type="single"
					value={filter}
					onValueChange={(v) => {
						const picked = oneOf(FILTER_VALUES, v);
						if (picked) saveUserSettings({ killmail_filter: picked });
					}}
				>
					<Select.Trigger class="w-52" data-testid="killmail-filter-select">
						{KILLMAIL_FILTERS.find((f) => f.value === filter)?.label}
					</Select.Trigger>
					<Select.Content>
						<Select.Group>
							{#each KILLMAIL_FILTERS as option (option.value)}
								<Select.Item value={option.value} label={option.label}>{option.label}</Select.Item>
							{/each}
						</Select.Group>
					</Select.Content>
				</Select.Root>
			{/snippet}
		</SettingRow>
	</Card.Content>
</Card.Root>
