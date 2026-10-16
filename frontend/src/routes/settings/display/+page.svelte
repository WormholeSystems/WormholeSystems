<script lang="ts">
	// How the site looks to you, on every map and every device. Per-map choices (threat
	// colours, placement, the background) are on each map's own display settings.
	import { toast } from 'svelte-sonner';

	import { invalidateAll } from '$app/navigation';
	import { page } from '$app/state';
	import { api, errorMessage } from '$lib/api/client';
	import type { UpdateUserPreferences } from '$lib/api/types/UpdateUserPreferences';
	import SettingRow from '$lib/components/settings/SettingRow.svelte';
	import * as Card from '$lib/components/ui/card';
	import { Switch } from '$lib/components/ui/switch';

	const preferences = $derived(page.data.preferences ?? null);
	let saving = $state(false);

	async function save(change: UpdateUserPreferences) {
		saving = true;
		try {
			await api.updatePreferences(change);
			// The layout loads them with every page, so that is what gets refreshed.
			await invalidateAll();
		} catch (err) {
			toast.error(errorMessage(err));
		} finally {
			saving = false;
		}
	}
</script>

<Card.Root>
	<Card.Header>
		<Card.Title>Display</Card.Title>
		<Card.Description>How WormholeSystems looks to you, on every map and device.</Card.Description>
	</Card.Header>
	<Card.Content class="flex flex-col py-0">
		<SettingRow
			id="effect-icons"
			label="Effect icons"
			description="Shows a wormhole's effect as the icon the game uses for it. Turn it off to see a coloured letter instead (B for Black Hole, P for Pulsar and so on)."
		>
			{#snippet control()}
				<Switch
					checked={preferences?.effect_icons ?? true}
					disabled={saving || !preferences}
					aria-label="Effect icons"
					data-testid="effect-icons-switch"
					onCheckedChange={(v) => save({ effect_icons: v })}
				/>
			{/snippet}
		</SettingRow>
	</Card.Content>
</Card.Root>
