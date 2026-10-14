<script lang="ts">
	// The set-destination / add-waypoint submenu pair, shared by the system and
	// destination menus. One entry per online pilot, and all of them at once.
	import MapPinIcon from '@lucide/svelte/icons/map-pin';
	import NavigationIcon from '@lucide/svelte/icons/navigation';
	import UsersIcon from '@lucide/svelte/icons/users';

	import * as ContextMenu from '$lib/components/ui/context-menu';
	import EveImage from '$lib/components/EveImage.svelte';
	import type { MapState } from '../../state/map-state.svelte';

	let { map, destinationId }: { map: MapState; destinationId: number } = $props();

	const online = $derived(map.characters.online);
</script>

{#snippet waypointItems(clearOthers: boolean)}
	{#if online.length === 0}
		<ContextMenu.Item disabled>No characters online</ContextMenu.Item>
	{:else}
		{#each online as c (c.character_id)}
			<ContextMenu.Item
				onSelect={() => map.waypoints.set(destinationId, c.character_id, clearOthers)}
			>
				<EveImage kind="character" id={c.character_id} size={32} class="size-5 rounded-lg" />
				{c.name}
			</ContextMenu.Item>
		{/each}
		{#if online.length > 1}
			<ContextMenu.Separator />
			<ContextMenu.Item onSelect={() => map.waypoints.setAll(destinationId, clearOthers)}>
				<UsersIcon class="size-4" />
				All Characters
			</ContextMenu.Item>
		{/if}
	{/if}
{/snippet}

<ContextMenu.Sub>
	<ContextMenu.SubTrigger data-testid="menu-set-destination">
		<NavigationIcon class="size-4" />
		Set destination
	</ContextMenu.SubTrigger>
	<ContextMenu.SubContent class="w-48" data-testid="destination-submenu">
		{@render waypointItems(true)}
	</ContextMenu.SubContent>
</ContextMenu.Sub>
<ContextMenu.Sub>
	<ContextMenu.SubTrigger data-testid="menu-add-waypoint">
		<MapPinIcon class="size-4" />
		Add waypoint
	</ContextMenu.SubTrigger>
	<ContextMenu.SubContent class="w-48" data-testid="waypoint-submenu">
		{@render waypointItems(false)}
	</ContextMenu.SubContent>
</ContextMenu.Sub>
