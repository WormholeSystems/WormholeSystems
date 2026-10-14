<script lang="ts">
	// What the chain calls its systems, and what that makes the bookmarks say.
	//
	// Mixed ownership: the names are the map's and Manager+, the clipboard toggle is yours.
	// Non-managers still see the names read-only.
	import { createQuery } from '@tanstack/svelte-query';
	import { page } from '$app/state';
	import {
		AUTOMATION,
		createUserSettingsWriter,
		settingBlocker,
		settingOn,
	} from '$lib/map/user-settings';
	import { api } from '$lib/api/client';
	import { apiAction } from '$lib/api/mutations';
	import { key, q } from '$lib/api/queries';
	import type { MapNaming } from '$lib/api/types/MapNaming';
	import type { MapView } from '$lib/api/types/MapView';
	import type { UpdateMapUserSettings } from '$lib/api/types/UpdateMapUserSettings';
	import SettingRow from '$lib/components/settings/SettingRow.svelte';
	import * as Card from '$lib/components/ui/card';
	import { Switch } from '$lib/components/ui/switch';
	import NamingCard from './NamingCard.svelte';
	import { atLeast } from '$lib/map/roles';

	let { data }: { data: { view: MapView } } = $props();

	const mapId = $derived(Number(page.params.id) || 0);
	const viewQuery = createQuery(() => ({ ...q.mapView(mapId), initialData: data.view }));
	const settingsQuery = createQuery(() => q.mapUserSettings(mapId));
	const settings = $derived(settingsQuery.data ?? null);

	const canManage = $derived(atLeast(viewQuery.data.role, 'manager'));
	const copy = AUTOMATION.copy_bookmark;
	const copyBlocked = $derived(settingBlocker(settings, 'copy_bookmark'));

	const settingsWriter = createUserSettingsWriter();
	const saveUserSettings = (patch: UpdateMapUserSettings) => settingsWriter.save(mapId, patch);
	const mapAct = apiAction(() => [key.mapView(mapId)]);

	function saveNaming(naming: MapNaming) {
		mapAct.mutate(() => api.updateMap({ map_id: mapId, naming }));
	}
</script>

<div class="flex flex-col gap-6">
	<NamingCard naming={viewQuery.data.map.naming} disabled={!canManage} onsave={saveNaming} />

	<Card.Root>
		<Card.Header>
			<Card.Title>Copying</Card.Title>
			<Card.Description>What reaches your clipboard. Yours alone, per map.</Card.Description>
		</Card.Header>
		<Card.Content class="flex flex-col py-0">
			<SettingRow
				id="copy-bookmark"
				label={copy.label}
				description={copy.body}
				disabled={copyBlocked !== null}
				blocked={copyBlocked ? `${copyBlocked} It is under Mapping.` : undefined}
			>
				{#snippet control()}
					<Switch
						checked={settingOn(settings, 'copy_bookmark')}
						disabled={copyBlocked !== null}
						aria-label={copy.label}
						onCheckedChange={(v) => saveUserSettings({ copy_bookmark: v })}
					/>
				{/snippet}
			</SettingRow>
		</Card.Content>
	</Card.Root>
</div>
