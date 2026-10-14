<script lang="ts">
	// The app-wide solar-system context menu: wrap any rendered system reference and
	// right-click (or long-press) it. Map-dependent items appear only when a MapState is
	// provided via the 'map-state' context, so the wrapper works on non-map surfaces too.
	import CompassIcon from '@lucide/svelte/icons/compass';
	import FlagIcon from '@lucide/svelte/icons/flag';
	import NavigationIcon from '@lucide/svelte/icons/navigation';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import RouteIcon from '@lucide/svelte/icons/route';
	import type { Snippet } from 'svelte';
	import { getMapContext } from './context';
	import ExternalLinksSubmenu from './ExternalLinksSubmenu.svelte';
	import WaypointSubmenus from './WaypointSubmenus.svelte';
	import { solarSystemId } from '$lib/map/system';

	import type { SystemSearchResult } from '$lib/api/types/SystemSearchResult';
	import * as ContextMenu from '$lib/components/ui/context-menu';
	import { isWormholeClass } from '$lib/map/classes';
	import { systemLinkGroups } from '$lib/map/external-links';

	let {
		system,
		class: cls = 'contents',
		children,
	}: {
		system: SystemSearchResult;
		/** Trigger element class; `contents` keeps the wrapped fragments in their row layout. */
		class?: string;
		children: Snippet;
	} = $props();

	// A getter, so the context value registered at init survives the page swapping maps.
	const getMap = getMapContext();
	const map = $derived(getMap?.());

	const canWrite = $derived(map !== undefined && map.canWrite);
	const placement = $derived(map?.systems.all.find((s) => solarSystemId(s) === system.id) ?? null);
	const watched = $derived(
		map?.watchlist.all.some((w) => w.solar_system_id === system.id) ?? false,
	);

	const groups = $derived(
		systemLinkGroups({
			solarSystemId: system.id,
			name: system.name,
			region: system.region,
			regionId: system.region_id,
			constellationId: system.constellation_id,
			isWormhole: isWormholeClass(system.wormhole_class_id),
		}),
	);
</script>

<ContextMenu.Root>
	<ContextMenu.Trigger class={cls} data-testid="system-menu-trigger" data-system-id={system.id}>
		{@render children()}
	</ContextMenu.Trigger>
	<ContextMenu.Content class="w-52" data-testid="system-menu">
		{#if map !== undefined && canWrite && (placement === null || !watched)}
			{#if placement === null}
				<ContextMenu.Item onclick={() => map.systems.add(system.id)} data-testid="menu-add-to-map">
					<PlusIcon class="size-4" />
					Add to map
				</ContextMenu.Item>
			{/if}
			{#if !watched}
				<ContextMenu.Item
					onclick={() => map.watchlist.add(system.id)}
					data-testid="menu-add-to-watchlist"
				>
					<EyeIcon class="size-4" />
					Add to watchlist
				</ContextMenu.Item>
			{/if}
			<ContextMenu.Separator />
		{/if}

		<ExternalLinksSubmenu {groups} />

		{#if map !== undefined}
			<WaypointSubmenus {map} destinationId={system.id} />
			<ContextMenu.Sub>
				<ContextMenu.SubTrigger data-testid="menu-route">
					<RouteIcon class="size-4" />
					Route planner
				</ContextMenu.SubTrigger>
				<ContextMenu.SubContent class="w-48">
					<ContextMenu.Item onclick={() => (map.route.fromId = system.id)}>
						<CompassIcon class="size-4" />
						Set as origin
					</ContextMenu.Item>
					<ContextMenu.Item onclick={() => (map.route.toId = system.id)}>
						<NavigationIcon class="size-4" />
						Set as destination
					</ContextMenu.Item>
				</ContextMenu.SubContent>
			</ContextMenu.Sub>

			{#if canWrite && placement !== null}
				<ContextMenu.Separator />
				<ContextMenu.Item
					onclick={() => map.systems.setRally(placement.id, !placement.is_rally)}
					data-testid="menu-rally"
				>
					<FlagIcon class="size-4" />
					{placement.is_rally ? 'Clear Rally Point' : 'Set as Rally Point'}
				</ContextMenu.Item>
			{/if}
		{/if}
	</ContextMenu.Content>
</ContextMenu.Root>
