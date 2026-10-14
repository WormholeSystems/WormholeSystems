<script lang="ts">
	// What the jump tracker does on your behalf, next to the toggle that turns it on.
	// Stored per user per map, like the route settings popover.
	import SettingsIcon from '@lucide/svelte/icons/settings-2';

	import { Button } from '$lib/components/ui/button';
	import * as Popover from '$lib/components/ui/popover';
	import { Switch } from '$lib/components/ui/switch';
	import type { UpdateMapUserSettings } from '$lib/api/types/UpdateMapUserSettings';
	import { trackedPilots } from '$lib/map/tracked-pilots';
	import type { MapState } from '../../state/map-state.svelte';

	let { map }: { map: MapState } = $props();

	const settings = $derived(map.userSettings);
	// Nothing here happens unless the tracker is running, so say so rather than offering
	// switches that quietly do nothing.
	const tracking = $derived(settings?.tracking_allowed ?? false);

	const OPTIONS: {
		key:
			| 'prompt_for_signature'
			| 'preselect_signature'
			| 'suggest_alias'
			| 'copy_bookmark'
			| 'follow_character';
		label: string;
		hint: string;
		/** A setting this one only means something under. */
		needs?: 'prompt_for_signature';
	}[] = [
		{
			key: 'prompt_for_signature',
			label: 'Ask which signature',
			hint: 'Otherwise the hole is mapped without a signature linked.',
		},
		{
			key: 'preselect_signature',
			label: 'Preselect the likeliest',
			hint: 'The prompt opens with the best match chosen, so Enter confirms it.',
			needs: 'prompt_for_signature',
		},
		{
			key: 'suggest_alias',
			label: 'Suggest an alias',
			hint: "Prefills the next name in the chain's sequence.",
		},
		{
			key: 'copy_bookmark',
			label: 'Copy the bookmark',
			hint: 'Puts the new bookmark on your clipboard once the jump is mapped.',
		},
		{
			key: 'follow_character',
			label: 'Follow me',
			hint: 'Selects the system you are in as you fly, so the cards keep up.',
		},
	];

	const pilots = $derived(map.characters.mine);
	const tracked = $derived(
		new Set(
			trackedPilots(pilots, settings?.tracked_character_ids ?? []).map((p) => p.character_id),
		),
	);

	function update(key: (typeof OPTIONS)[number]['key'], value: boolean) {
		const patch: UpdateMapUserSettings = {};
		patch[key] = value;
		map.patchUserSettings(patch);
	}

	function togglePilot(characterId: number, on: boolean) {
		const next = new Set(tracked);
		if (on) next.add(characterId);
		else next.delete(characterId);
		map.patchUserSettings({ tracked_character_ids: [...next] });
	}
</script>

<Popover.Root>
	<Popover.Trigger>
		{#snippet child({ props })}
			<Button
				{...props}
				variant="ghost"
				size="icon"
				class="size-7 text-muted-foreground/50"
				aria-label="Jump tracking settings"
				data-testid="tracking-settings"
			>
				<SettingsIcon />
			</Button>
		{/snippet}
	</Popover.Trigger>
	<Popover.Content class="w-72" align="start">
		<div class="flex flex-col gap-3">
			<div class="flex flex-col gap-0.5">
				<span class="text-xs font-medium">Jump tracking</span>
				<span class="text-[11px] text-muted-foreground">
					{tracking
						? 'Flying through an unmapped hole builds it on the map.'
						: 'Turn on Share location to build the map as you fly.'}
				</span>
			</div>
			{#each OPTIONS as option (option.key)}
				<label class="flex items-start justify-between gap-3" for="tracking-{option.key}">
					<span class="flex flex-col gap-0.5">
						<span class="text-xs">{option.label}</span>
						<span class="text-[11px] text-muted-foreground">{option.hint}</span>
					</span>
					<Switch
						id="tracking-{option.key}"
						disabled={!tracking || (option.needs !== undefined && !settings?.[option.needs])}
						checked={settings?.[option.key] ?? false}
						onCheckedChange={(v) => update(option.key, v)}
						data-testid="setting-{option.key}"
					/>
				</label>
			{/each}
			{#if pilots.length > 1}
				<div class="flex flex-col gap-0.5 border-t pt-3">
					<span class="text-xs font-medium">Pilots mapping here</span>
					<span class="text-[11px] text-muted-foreground">
						Their jumps build this map. With none chosen, the pilot you are acting as does.
					</span>
				</div>
				{#each pilots as pilot (pilot.character_id)}
					<label
						class="flex items-center justify-between gap-3"
						for="tracking-pilot-{pilot.character_id}"
					>
						<span class="text-xs">{pilot.name}</span>
						<Switch
							id="tracking-pilot-{pilot.character_id}"
							disabled={!tracking}
							checked={tracked.has(pilot.character_id)}
							onCheckedChange={(v) => togglePilot(pilot.character_id, v)}
							data-testid="tracking-pilot-{pilot.character_id}"
						/>
					</label>
				{/each}
			{/if}
		</div>
	</Popover.Content>
</Popover.Root>
