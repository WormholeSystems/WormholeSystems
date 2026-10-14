<script lang="ts">
	// What can be done with a solar system from any menu that names one: put it on the map or
	// the watchlist, look it up elsewhere, send a pilot there, route to it, and rally on it.
	// The right-click menu around panel rows and the map node's own menu both render this, so
	// the two cannot drift apart again. Rendered inside either kind of bits-ui menu: the items
	// are the same primitives under a context menu and a dropdown.
	import BanIcon from '@lucide/svelte/icons/ban';
	import CompassIcon from '@lucide/svelte/icons/compass';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import FlagIcon from '@lucide/svelte/icons/flag';
	import NavigationIcon from '@lucide/svelte/icons/navigation';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import RouteIcon from '@lucide/svelte/icons/route';

	import type { SystemSearchResult } from '$lib/api/types/SystemSearchResult';
	import * as ContextMenu from '$lib/components/ui/context-menu';
	import { isWormholeClass } from '$lib/map/classes';
	import { systemLinkGroups } from '$lib/map/external-links';
	import { solarSystemId } from '$lib/map/system';
	import type { MapState } from '../../state/map-state.svelte';
	import ExternalLinksSubmenu from './ExternalLinksSubmenu.svelte';
	import WaypointSubmenus from './WaypointSubmenus.svelte';

	let { map, system }: { map: MapState; system: SystemSearchResult } = $props();

	const canWrite = $derived(map.canWrite);
	const placement = $derived(map.systems.all.find((s) => solarSystemId(s) === system.id) ?? null);
	const watched = $derived(map.watchlist.all.some((w) => w.solar_system_id === system.id));
	const ignored = $derived(map.ignored.has(system.id));
	// ESI refuses a destination in wormhole space, so the waypoint menus only offer k-space.
	const wormhole = $derived(isWormholeClass(system.wormhole_class_id));

	const groups = $derived(
		systemLinkGroups({
			solarSystemId: system.id,
			name: system.name,
			region: system.region,
			regionId: system.region_id,
			constellationId: system.constellation_id,
			isWormhole: wormhole,
		}),
	);
</script>

{#if canWrite && (placement === null || !watched)}
	{#if placement === null}
		<ContextMenu.Item onSelect={() => map.systems.add(system.id)} data-testid="menu-add-to-map">
			<PlusIcon class="size-4" />
			Add to map
		</ContextMenu.Item>
	{/if}
	{#if !watched}
		<ContextMenu.Item
			onSelect={() => map.watchlist.add(system.id)}
			data-testid="menu-add-to-watchlist"
		>
			<EyeIcon class="size-4" />
			Add to watchlist
		</ContextMenu.Item>
	{/if}
	<ContextMenu.Separator />
{/if}

<ExternalLinksSubmenu {groups} />

{#if !wormhole}
	<WaypointSubmenus {map} destinationId={system.id} />
{/if}

<ContextMenu.Sub>
	<ContextMenu.SubTrigger data-testid="menu-route">
		<RouteIcon class="size-4" />
		Route planner
	</ContextMenu.SubTrigger>
	<ContextMenu.SubContent class="w-52" data-testid="route-submenu">
		<ContextMenu.Item onSelect={() => (map.route.fromId = system.id)}>
			<CompassIcon class="size-4" />
			Set as origin
		</ContextMenu.Item>
		<ContextMenu.Item onSelect={() => (map.route.toId = system.id)}>
			<NavigationIcon class="size-4" />
			Set as destination
		</ContextMenu.Item>
		{#if map.canManage}
			<ContextMenu.Separator />
			<ContextMenu.Item
				data-testid="ignore-for-everyone"
				onSelect={() => (ignored ? map.ignored.remove(system.id) : map.ignored.add(system.id))}
			>
				<BanIcon class="size-4" />
				{ignored ? 'Stop ignoring for everyone' : 'Ignore for everyone'}
			</ContextMenu.Item>
		{/if}
	</ContextMenu.SubContent>
</ContextMenu.Sub>

{#if canWrite && placement !== null}
	<ContextMenu.Separator />
	<ContextMenu.Item
		onSelect={() => map.systems.setRally(placement.id, !placement.is_rally)}
		data-testid="menu-rally"
	>
		<FlagIcon class="size-4" />
		{placement.is_rally ? 'Clear Rally Point' : 'Set as Rally Point'}
	</ContextMenu.Item>
{/if}
