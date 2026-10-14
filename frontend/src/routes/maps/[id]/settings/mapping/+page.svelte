<script lang="ts">
	// What the map does for you while you fly. Location sharing gates the rest, and is shown
	// as gating it rather than leaving dead toggles around.
	//
	// The scanning card is the map's setting, not this viewer's: an unmapped hole put on the
	// map is a node everyone sees.
	import { createQuery } from '@tanstack/svelte-query';
	import { page } from '$app/state';
	import {
		AUTOMATION,
		automationPatch,
		createUserSettingsWriter,
		settingBlocker,
		settingOn,
	} from '$lib/map/user-settings';
	import { api } from '$lib/api/client';
	import { apiAction } from '$lib/api/mutations';
	import { key, q } from '$lib/api/queries';
	import type { MapView } from '$lib/api/types/MapView';
	import type { UpdateMapUserSettings } from '$lib/api/types/UpdateMapUserSettings';
	import SettingRow from '$lib/components/settings/SettingRow.svelte';
	import * as Card from '$lib/components/ui/card';
	import { Switch } from '$lib/components/ui/switch';
	import { atLeast } from '$lib/map/roles';
	import { LOCATION_SCOPE } from '$lib/esi/scopes';

	let { data }: { data: { view: MapView } } = $props();

	const mapId = $derived(Number(page.params.id) || 0);
	const scopesQuery = createQuery(() => q.myScopes());
	const scopes = $derived(scopesQuery.data ?? []);
	const settingsQuery = createQuery(() => q.mapUserSettings(mapId));
	const settings = $derived(settingsQuery.data ?? null);
	const viewQuery = createQuery(() => ({ ...q.mapView(mapId), initialData: data.view }));

	const hasLocation = $derived(scopes.some((s) => s.scope === LOCATION_SCOPE && s.granted));
	const MAPPING = (
		['tracking_allowed', 'prompt_for_signature', 'preselect_signature', 'suggest_alias'] as const
	).map((k) => AUTOMATION[k]);
	const canManage = $derived(atLeast(viewQuery.data.role, 'manager'));
	const ghosting = $derived(viewQuery.data.map.ghost_unlinked_wormholes);

	const settingsWriter = createUserSettingsWriter();
	const saveUserSettings = (patch: UpdateMapUserSettings) => settingsWriter.save(mapId, patch);
	const mapAct = apiAction(() => [key.mapView(mapId)]);

	function updateMap(ghost: boolean) {
		mapAct.mutate(() => api.updateMap({ map_id: mapId, ghost_unlinked_wormholes: ghost }));
	}
</script>

<div class="flex flex-col gap-6">
	<Card.Root>
		<Card.Header>
			<Card.Title>Scanning</Card.Title>
			<Card.Description>What a pasted scan puts on the map, for everyone on it.</Card.Description>
		</Card.Header>
		<Card.Content class="flex flex-col py-0">
			<SettingRow
				id="ghost-unlinked-wormholes"
				label="Put unmapped wormholes on the map"
				description="A pasted wormhole signature that is not on the map yet gets a node hanging off the system it was scanned in, with no system behind it. Lay the chain out and name it before anyone flies it; assign the system from the node's menu once someone has."
				disabled={!canManage}
				blocked={canManage ? undefined : 'Only a manager can change this.'}
			>
				{#snippet control()}
					<Switch
						checked={ghosting}
						disabled={!canManage}
						aria-label="Put unmapped wormholes on the map"
						onCheckedChange={(v) => updateMap(v)}
					/>
				{/snippet}
			</SettingRow>
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>Mapping</Card.Title>
			<Card.Description>
				What happens automatically as you fly the chain. Yours alone, per map.
			</Card.Description>
		</Card.Header>
		<Card.Content class="flex flex-col py-0">
			{#each MAPPING as setting (setting.key)}
				{@const blocked = settingBlocker(settings, setting.key, hasLocation)}
				<SettingRow
					id={setting.key.replaceAll('_', '-')}
					label={setting.label}
					description={setting.body}
					disabled={blocked !== null}
					blocked={blocked ?? undefined}
				>
					{#snippet control()}
						<Switch
							checked={settingOn(settings, setting.key, hasLocation)}
							disabled={blocked !== null}
							aria-label={setting.label}
							onCheckedChange={(v) => saveUserSettings(automationPatch(setting.key, v))}
						/>
					{/snippet}
				</SettingRow>
			{/each}
		</Card.Content>
	</Card.Root>
</div>
